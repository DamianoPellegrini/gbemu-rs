use gbemu::{cartridge::CartridgeHolder, cpu::Cpu};
fn main() {
    env_logger::init();

    let game = std::fs::read("rom/pkmn_yel.gb").expect("Failed to read game file.");
    let mut gb = gbemu::GameBoy::new(&game);

    let cart_header = gb.cartridge_header();
    log::info!("Game loaded!");
    log::info!("Game Info: {:#?}.", cart_header);

    // 0x603C

    let mut start = std::time::Instant::now();
    let mut delta_time = std::time::Duration::from_secs_f64(0.0);
    loop {
        gb.tick(delta_time.as_secs_f64());

        delta_time = start.elapsed();
        start = std::time::Instant::now();
    }
}
