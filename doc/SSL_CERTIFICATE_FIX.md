# SSL 证书路径自动检测修复

## 问题描述

在 Mac ARM 环境下通过 GitHub Actions 打包得到的 app，OpenSSL 路径与用户 Mac 不一致，导致无法找到证书，报错：

```
unable to get local issuer certificate
```

## 问题原因

通过 GitHub Actions 在特定环境下编译打包时，OpenSSL 使用的证书路径是打包环境的路径（例如 `/opt/homebrew/etc/openssl@3/cert.pem`），但用户 Mac 上的证书可能在不同的位置（例如 `/usr/local/etc/openssl/cert.pem` 或 `/etc/ssl/cert.pem`），导致程序运行时找不到证书。

## 解决方案

在程序启动时，使用 `openssl-probe` crate 自动探测系统中的 SSL 证书路径，并设置相应的环境变量。

### 修改内容

1. **添加依赖**（`crates/openconnect-core/Cargo.toml`）：
   ```toml
   openssl-probe = "0.1.5"
   ```

2. **在 VpnClient 初始化时自动探测证书路径**（`crates/openconnect-core/src/lib.rs`）：
   ```rust
   fn new(config: Config, callbacks: EventHandlers) -> OpenconnectResult<Arc<Self>> {
       // 在初始化 SSL 前，先探测并配置证书路径
       // 这样可以确保在不同环境（包括 GitHub Actions 打包的版本）都能找到正确的证书
       openssl_probe::init_ssl_cert_env_vars();
       
       // 记录探测到的证书路径，方便调试
       if let Ok(cert_file) = std::env::var("SSL_CERT_FILE") {
           tracing::debug!("SSL_CERT_FILE set to: {}", cert_file);
       }
       if let Ok(cert_dir) = std::env::var("SSL_CERT_DIR") {
           tracing::debug!("SSL_CERT_DIR set to: {}", cert_dir);
       }
       
       // ... 其余初始化代码
   }
   ```

### 工作原理

`openssl-probe::init_ssl_cert_env_vars()` 会自动扫描以下常见的证书路径：

**macOS 常见路径：**
- `/etc/ssl/cert.pem`
- `/etc/ssl/certs/`
- `/usr/local/etc/openssl/cert.pem`
- `/usr/local/etc/openssl@1.1/cert.pem`
- `/usr/local/etc/openssl@3/cert.pem`
- `/opt/homebrew/etc/openssl/cert.pem`
- `/opt/homebrew/etc/openssl@1.1/cert.pem`
- `/opt/homebrew/etc/openssl@3/cert.pem`
- `/System/Library/OpenSSL/`

**Linux 常见路径：**
- `/etc/ssl/certs/ca-certificates.crt`
- `/etc/pki/tls/certs/ca-bundle.crt`
- `/etc/ssl/cert.pem`
- `/etc/ssl/certs/`

找到存在的证书文件或目录后，会自动设置 `SSL_CERT_FILE` 和 `SSL_CERT_DIR` 环境变量，OpenSSL 会使用这些环境变量来定位证书。

## 效果

- ✅ 在任何 Mac（Intel 或 ARM）上运行时，程序都能自动找到系统证书
- ✅ 通过 GitHub Actions 打包的版本不再受限于打包环境的证书路径
- ✅ 减少了 "unable to get local issuer certificate" 错误
- ✅ 提升了跨环境的兼容性和可移植性

## 调试

如果仍然遇到证书问题，可以：

1. 查看日志中的证书路径信息：
   ```
   SSL_CERT_FILE set to: /opt/homebrew/etc/openssl@3/cert.pem
   SSL_CERT_DIR set to: /etc/ssl/certs
   ```

2. 手动检查系统中的证书文件：
   ```bash
   # macOS (Homebrew)
   ls -la /opt/homebrew/etc/openssl@3/cert.pem
   ls -la /usr/local/etc/openssl/cert.pem
   
   # macOS (系统)
   ls -la /etc/ssl/cert.pem
   
   # 查看当前 OpenSSL 配置
   openssl version -d
   ```

3. 验证证书是否有效：
   ```bash
   openssl verify -CAfile /path/to/cert.pem /path/to/cert.pem
   ```

## 相关链接

- [openssl-probe crate](https://crates.io/crates/openssl-probe)
- [OpenSSL Certificate Locations](https://www.openssl.org/docs/man1.1.1/man1/c_rehash.html)

