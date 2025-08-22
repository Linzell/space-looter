//! Space Looter - 3D Isometric Dice RPG
//!
//! Main executable entry point for the RPG demo

use space_looter::run;

fn main() {
    println!("\n🎮 Welcome to Space Looter - 3D Isometric Dice RPG!");
    println!("═══════════════════════════════════════════════════");
    println!("🚀 A space exploration RPG with dice-based mechanics");
    println!();
    println!("📋 CONTROLS:");
    println!("  🎲 SPACE       - Roll dice for actions");
    println!("  🏃 WASD/Arrows - Move around the map");
    println!("  🏠 B           - Base management");
    println!("  📜 Q           - Quest log");
    println!("  🎒 I           - Inventory");
    println!("  ⏸️  ESC         - Pause/Resume");
    println!("  ▶️  ENTER       - Start game (from menu)");
    println!();
    println!("🎯 RPG FEATURES:");
    println!("  ✨ Dice-based exploration and combat");
    println!("  🏗️  Base building and upgrades");
    println!("  📈 Character progression");
    println!("  🗺️  Procedural world generation");
    println!("  ⚔️  Turn-based encounters");
    println!("  💰 Resource gathering");
    println!();
    println!("🎲 Starting your adventure...\n");

    // Run the RPG
    run();
}
