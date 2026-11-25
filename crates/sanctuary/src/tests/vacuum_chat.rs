use glam::{Quat, Vec3};
use world::{
    BodyId, CelestialSystem, CompanionPortal, CompanionPresence, CosmicFrame, CraftIdentity,
    PlanetaryWorld, PresenceAttire, PresenceKinematics, SolarProportions, VesselBerth,
    WorldEarthworks,
};
use crate::{
    ChatLocus, ChatMedium, HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost,
    SuitTransceiverMedium, HELMET_TOUCH_METERS, SUIT_RADIO_METERS,
};

fn squire_at(pos: Vec3) -> CompanionPresence {
    CompanionPresence::new(
        0,
        PresenceKinematics::new(pos, Quat::IDENTITY, Quat::IDENTITY, (1.45, 0.0)),
        (0.0, 0.0, 0.0, 0.0),
        PresenceAttire::squire(),
    )
}

#[test]
fn test_chat_medium_enforces_vacuum_silence_eva_radio_cabin_intercom_and_helmet_touch() {
    let medium = SuitTransceiverMedium::cosmos();
    let ship_a = CraftIdentity::new(0xA11C_E001);
    let ship_b = CraftIdentity::new(0xB0B0_0002);

    // 1. Atmosphere (`Earth`, `Mars`, `Titan`) carries open acoustic waves up to 96 m, even with radio off:
    for body in [BodyId::EARTH, BodyId::MARS, BodyId::TITAN] {
        let f = CosmicFrame::Body(body);
        let spk = ChatLocus::new(f, VesselBerth::planetary(), Vec3::ZERO, false);
        let near = ChatLocus::new(f, VesselBerth::planetary(), Vec3::new(82.0, 0.0, 0.0), false);
        let far = ChatLocus::new(f, VesselBerth::planetary(), Vec3::new(112.0, 0.0, 0.0), true);
        assert!(medium.carries(spk, near), "{body:?} atmosphere must carry open voice at 82 m");
        assert!(!medium.carries(spk, far), "{body:?} voice must drop to 0 beyond 96 m");
    }

    // 2. Airless Vacuum (`Luna`, `BeltAsteroid`, `DeepSpace`): 0 m open sound, <= 32 m EVA suit VHF, <= 1.85 m helmet touch:
    let luna = CosmicFrame::Body(BodyId::LUNA);
    let radio_spk = ChatLocus::new(luna, VesselBerth::planetary(), Vec3::ZERO, true);
    let at_50m = ChatLocus::new(luna, VesselBerth::planetary(), Vec3::new(50.0, 0.0, 0.0), true);
    let at_24m = ChatLocus::new(luna, VesselBerth::planetary(), Vec3::new(SUIT_RADIO_METERS - 6.0, 0.0, 0.0), true);
    assert!(!medium.carries(radio_spk, at_50m), "Airless Luna must carry 0 open sound at 50 m");
    assert!(medium.carries(radio_spk, at_24m), "EVA suit radio must carry within 32 m");

    // Radio-silent (`~`) stealth mode in vacuum: silent at 5 m, audible ONLY via visor-to-visor helmet touch (<= 1.85 m):
    let stealth_spk = ChatLocus::new(luna, VesselBerth::planetary(), Vec3::ZERO, false);
    let at_5m = ChatLocus::new(luna, VesselBerth::planetary(), Vec3::new(5.0, 0.0, 0.0), true);
    let visor_touch = ChatLocus::new(luna, VesselBerth::planetary(), Vec3::new(HELMET_TOUCH_METERS - 0.25, 0.0, 0.0), false);
    assert!(!medium.carries(stealth_spk, at_5m), "Radio-silent vacuum whisper must emit 0 radio at 5 m");
    assert!(medium.carries(stealth_spk, visor_touch), "Helmet-to-helmet visor contact must carry voice in hard vacuum");

    // 3. Pressurized `CabinIntercomMedium`: carries across the same ship deck regardless of radio silence, isolated from other ships:
    let deep = CosmicFrame::DeepSpace([12, -4, 9]);
    let bridge = ChatLocus::new(deep, VesselBerth::aboard(ship_a), Vec3::ZERO, false);
    let aft_bay = ChatLocus::new(deep, VesselBerth::aboard(ship_a), Vec3::new(0.0, 0.0, -45.0), false);
    let rival_ship = ChatLocus::new(deep, VesselBerth::aboard(ship_b), Vec3::new(0.0, 0.0, -45.0), true);
    assert!(medium.carries(bridge, aft_bay), "Pressurized ship hull must conduct cabin intercom");
    assert!(!medium.carries(bridge, rival_ship), "Separate ship in vacuum must not overhear radio-silent bridge");
}

#[test]
fn test_live_udp_lunar_vacuum_chat_drops_beyond_radio_and_carries_helmet_contact_whisper() {
    let prop = SolarProportions::compressed(32, 0.01);
    let raw_host = SanctuaryHost::bind_cosmos("127.0.0.1:0", CelestialSystem::barren_heliosphere(prop, 1.0), prop)
        .expect("bind cosmos host");
    let addr = raw_host.local_addr().expect("host addr");
    let mut host = HostedPlanet::spawning(raw_host);

    let mut speaker = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("connect speaker");
    let mut close_eva = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("connect close EVA");
    let mut far_crater = RemotePlanet::connect_cosmos(JoinTarget::Direct(addr), prop, 1.0).expect("connect 55m crater peer");

    let luna_center = speaker.body_center(BodyId::LUNA).expect("Luna center");
    let _ = speaker.reconcile_cosmic_frame(luna_center + Vec3::Y * 8.0, Vec3::ZERO);
    let _ = close_eva.reconcile_cosmic_frame(luna_center + Vec3::Y * 8.0, Vec3::ZERO);
    let _ = far_crater.reconcile_cosmic_frame(luna_center + Vec3::Y * 8.0, Vec3::ZERO);
    assert_eq!(speaker.cosmic_frame(), CosmicFrame::Body(BodyId::LUNA));

    // Place speaker at (0, 8, 0), close_eva at 12 m (inside 32 m EVA radio), far_crater at 55 m (audible on Earth, silent on Luna!)
    speaker.sync_presence(squire_at(Vec3::new(0.0, 8.0, 0.0)));
    close_eva.sync_presence(squire_at(Vec3::new(12.0, 8.0, 0.0)));
    far_crater.sync_presence(squire_at(Vec3::new(55.0, 8.0, 0.0)));
    for _ in 0..8 {
        let _ = speaker.step_surface(0.02);
        let _ = close_eva.step_surface(0.02);
        let _ = far_crater.step_surface(0.02);
        let _ = host.step_surface(0.02);
    }

    speaker.utter_nearby("Check lunar regolith seam");
    for _ in 0..6 {
        let _ = speaker.step_surface(0.02);
        let _ = host.step_surface(0.02);
        let _ = close_eva.step_surface(0.02);
        let _ = far_crater.step_surface(0.02);
    }
    let (mut close_inbox, mut far_inbox) = (Vec::new(), Vec::new());
    close_eva.drain_utterances(&mut close_inbox);
    far_crater.drain_utterances(&mut far_inbox);
    assert_eq!(close_inbox.len(), 1, "12 m lunar EVA suit transceiver must receive voice");
    assert!(far_inbox.is_empty(), "55 m airless lunar crater observer must receive 0 bytes (vacuum silence)");

    // Radio-silent (`~`) whisper at 12 m -> 0 bytes! Step visor-to-visor (1.2 m) -> received via HelmetContactMedium!
    speaker.utter_nearby("~pirate skiff overhead, cut suit radio");
    for _ in 0..6 {
        let _ = speaker.step_surface(0.02);
        let _ = host.step_surface(0.02);
        let _ = close_eva.step_surface(0.02);
    }
    close_inbox.clear();
    close_eva.drain_utterances(&mut close_inbox);
    assert!(close_inbox.is_empty(), "Radio-silent `~` message must not broadcast over 12 m EVA radio");

    close_eva.sync_presence(squire_at(Vec3::new(1.2, 8.0, 0.0)));
    for _ in 0..6 {
        let _ = close_eva.step_surface(0.02);
        let _ = host.step_surface(0.02);
    }
    speaker.utter_nearby("~helmets touching, copy?");
    for _ in 0..6 {
        let _ = speaker.step_surface(0.02);
        let _ = host.step_surface(0.02);
        let _ = close_eva.step_surface(0.02);
    }
    close_eva.drain_utterances(&mut close_inbox);
    assert_eq!(close_inbox.len(), 1, "1.2 m visor-to-visor helmet touch must transmit `~` whisper in vacuum");
}
