use world::SolarProportions;
use crate::cosmos_scale::{ArgumentCosmos, CosmosScale, FileCosmos, MeasuredCosmos};
use crate::server_scale::ScaledCube;
use crate::toml_document::TomlDocument;

#[test]
fn test_cosmos_decorator_chain_prioritizes_cli_over_client_and_server_toml() {
    let base = Box::new(MeasuredCosmos::new(Box::new(ScaledCube::new(2))));
    let server_doc = TomlDocument::new(
        "[cosmos]\nbody_scale = 0.5\nsun_scale = 0.8\ndistance_scale = 0.1\nasteroid_density = 0.4\n".to_string(),
    );
    let client_doc = TomlDocument::new("[cosmos]\nasteroid_density = 1.75\n".to_string());
    let s_layer = Box::new(FileCosmos::from_document(&server_doc, "server.toml", base));
    let c_layer = Box::new(FileCosmos::from_document(&client_doc, "client.toml", s_layer));
    let args = vec![
        "client".to_string(),
        "--distance-scale".to_string(),
        "0.25".to_string(),
    ];
    let cli_layer = ArgumentCosmos::from_arguments(&args, c_layer);
    let p = cli_layer.proportions();
    assert_eq!(p.earth_blocks(), 256);
    assert!((p.body_scale() - 0.5).abs() < 1e-5);
    assert!((p.sun_scale() - 0.8).abs() < 1e-5);
    assert!((p.distance_scale() - 0.25).abs() < 1e-5);
    assert!((p.belt_density() - 1.75).abs() < 1e-5);
}

#[test]
fn test_sanctuary_handshake_transmits_solar_proportions_to_guest() {
    let custom = SolarProportions::true_scale(32)
        .with_body_scale(0.6)
        .with_sun_scale(1.4)
        .with_distance_scale(0.08)
        .with_belt_density(1.5);
    let world = world::CelestialSystem::barren_authoritative(32, 1.0);
    let mut host = sanctuary::SanctuaryHost::bind_cosmos("127.0.0.1:0", world, custom)
        .expect("bind cosmos host");
    let addr = host.local_addr().expect("host addr");
    let mut guest = sanctuary::RemoteLink::bind(sanctuary::JoinTarget::Direct(addr))
        .expect("bind guest");
    let mut received = None;
    for _ in 0..20 {
        let _ = host.tick(0.02);
        if let Some(prop) = guest.step_cosmos_handshake(0.02) {
            received = Some(prop);
            break;
        }
    }
    assert_eq!(received, Some(custom), "Guest must receive host SolarProportions over LWEL handshake");
}
