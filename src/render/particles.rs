use crate::resources::EntityType;

/// Maximum number of particles alive at once.
const MAX_PARTICLES: usize = 50;

/// A single visual particle.
#[derive(Clone, Debug)]
pub struct Particle {
    pub x: usize,
    pub y: usize,
    pub glyph: char,
    pub fg: (u8, u8, u8),
    pub frames_remaining: u8,
    pub drift_dx: i8,
    pub drift_dy: i8,
    pub fade: bool,
}

/// Manages all active particles.
pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem {
            particles: Vec::new(),
        }
    }

    /// Spawn a particle. Silently drops if at capacity.
    pub fn spawn(&mut self, particle: Particle) {
        if self.particles.len() < MAX_PARTICLES {
            self.particles.push(particle);
        }
    }

    /// Tick all particles: move, decrement frames, remove expired.
    pub fn tick(&mut self) {
        for p in &mut self.particles {
            p.frames_remaining = p.frames_remaining.saturating_sub(1);
            if p.drift_dx != 0 || p.drift_dy != 0 {
                p.x = (p.x as isize + p.drift_dx as isize).max(0) as usize;
                p.y = (p.y as isize + p.drift_dy as isize).max(0) as usize;
            }
            if p.fade && p.frames_remaining > 0 {
                // Dim the color slightly each frame
                p.fg.0 = p.fg.0.saturating_sub(15);
                p.fg.1 = p.fg.1.saturating_sub(15);
                p.fg.2 = p.fg.2.saturating_sub(15);
            }
        }
        self.particles.retain(|p| p.frames_remaining > 0);
    }

    /// Get the topmost particle at a position, if any.
    pub fn get_at(&self, x: usize, y: usize) -> Option<&Particle> {
        self.particles.iter().rev().find(|p| p.x == x && p.y == y)
    }

    /// Get all active particles.
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// Clear all particles.
    pub fn clear(&mut self) {
        self.particles.clear();
    }
}

/// Spawn machine-specific particles during processing.
pub fn spawn_machine_particles(system: &mut ParticleSystem, entity_type: EntityType, x: usize, y: usize) {
    match entity_type {
        EntityType::Smelter => {
            // Gold sparks above
            system.spawn(Particle {
                x, y: y.saturating_sub(1),
                glyph: '*',
                fg: (200, 180, 60),
                frames_remaining: 3,
                drift_dx: 0, drift_dy: 0,
                fade: true,
            });
            // Orange ember
            system.spawn(Particle {
                x, y: y.saturating_sub(1),
                glyph: '.',
                fg: (220, 140, 40),
                frames_remaining: 2,
                drift_dx: 0, drift_dy: -1,
                fade: true,
            });
        }
        EntityType::Kiln => {
            // Smoke drifting up
            system.spawn(Particle {
                x, y: y.saturating_sub(1),
                glyph: '~',
                fg: (160, 150, 130),
                frames_remaining: 4,
                drift_dx: 0, drift_dy: -1,
                fade: true,
            });
        }
        EntityType::ChemicalPlant => {
            // Green bubbles
            system.spawn(Particle {
                x, y,
                glyph: '\u{00B0}', // degree sign as bubble
                fg: (80, 200, 100),
                frames_remaining: 3,
                drift_dx: 0, drift_dy: -1,
                fade: false,
            });
        }
        EntityType::Boiler => {
            // Steam
            system.spawn(Particle {
                x, y: y.saturating_sub(1),
                glyph: '\u{2591}', // light shade
                fg: (200, 210, 220),
                frames_remaining: 4,
                drift_dx: 0, drift_dy: -1,
                fade: true,
            });
        }
        EntityType::CoolantProcessor => {
            // Frost
            system.spawn(Particle {
                x, y,
                glyph: '*',
                fg: (150, 200, 255),
                frames_remaining: 3,
                drift_dx: 0, drift_dy: 0,
                fade: true,
            });
        }
        EntityType::QuantumLab => {
            // Purple/blue flicker
            let fg = if (x + y) % 2 == 0 {
                (150, 80, 220)
            } else {
                (80, 120, 255)
            };
            system.spawn(Particle {
                x, y,
                glyph: '\u{00B7}', // middle dot
                fg,
                frames_remaining: 2,
                drift_dx: 0, drift_dy: 0,
                fade: false,
            });
        }
        EntityType::FusionReactor => {
            // Plasma glow
            system.spawn(Particle {
                x, y,
                glyph: '\u{25E6}', // white bullet
                fg: (255, 220, 100),
                frames_remaining: 3,
                drift_dx: 0, drift_dy: 0,
                fade: true,
            });
        }
        EntityType::SingularityLab => {
            // Void sparks
            system.spawn(Particle {
                x, y: y.saturating_sub(1),
                glyph: '/',
                fg: (180, 60, 220),
                frames_remaining: 2,
                drift_dx: 0, drift_dy: 0,
                fade: true,
            });
        }
        _ => {}
    }
}
