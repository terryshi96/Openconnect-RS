use openconnect_core::{
    config::{ConfigBuilder, EntrypointBuilder, LogLevel},
    events::EventHandlers,
    protocols::get_anyconnect_protocol,
    Connectable, VpnClient,
};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::from_path(".env.local").unwrap();
    // 注意：VpnClient::new() 会自动探测并设置证书路径，无需手动配置
    // 下面这行是为了避免某些系统上的 openssl 配置问题
    env::set_var("OPENSSL_CONF", "/dev/null");

    let protocol = get_anyconnect_protocol();

    let config = ConfigBuilder::default().loglevel(LogLevel::Info).build()?;

    let event_handlers = EventHandlers::default();

    let client = VpnClient::new(config, event_handlers)?;

    let entrypoint = EntrypointBuilder::new()
        .server(&env::var("VPN_SERVER").unwrap())
        .username(&env::var("VPN_USERNAME").unwrap())
        .password(&env::var("VPN_PASSWORD").unwrap())
        .protocol(protocol)
        .enable_udp(true)
        .accept_insecure_cert(true)
        .build()?;

    client.init_connection(entrypoint)?;
    client.run_loop()?;

    Ok(())
}
