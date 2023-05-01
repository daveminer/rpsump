use actix_governor::{
    governor::{clock::QuantaInstant, middleware::NoOpMiddleware},
    Governor, GovernorConfigBuilder, PeerIpKeyExtractor,
};
pub fn new(
    per_second: u64,
    burst_size: u32,
) -> Governor<PeerIpKeyExtractor, NoOpMiddleware<QuantaInstant>> {
    let config = GovernorConfigBuilder::default()
        .per_second(per_second)
        .burst_size(burst_size)
        .finish()
        .expect("Could not build rate limiter config.");

    Governor::new(&config)
}
