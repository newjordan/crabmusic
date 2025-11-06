// Floppy disk save animation for git commit

use super::Animation;
use crate::braille::{BrailleGrid, Color};
use std::time::Duration;

/// Save/commit animation with floppy disk
pub struct SaveAnimation {
    elapsed: Duration,
    duration: Duration,
    phase: Phase,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Phase {
    Appearing,
    ProgressBar,
    Checkmark,
    Done,
}

impl SaveAnimation {
    pub fn new(duration: Duration) -> Self {
        Self {
            elapsed: Duration::ZERO,
            duration,
            phase: Phase::Appearing,
        }
    }
}

impl Default for SaveAnimation {
    fn default() -> Self {
        Self::new(Duration::from_millis(1500))
    }
}

impl Animation for SaveAnimation {
    fn update(&mut self, delta_time: Duration) -> bool {
        self.elapsed += delta_time;

        // Phase transitions
        let progress = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
        self.phase = if progress < 0.3 {
            Phase::Appearing
        } else if progress < 0.7 {
            Phase::ProgressBar
        } else if progress < 0.9 {
            Phase::Checkmark
        } else if progress < 1.0 {
            Phase::Done
        } else {
            return false; // Animation complete
        };

        true
    }

    fn render(&self, grid: &mut BrailleGrid) {
        let center_x = grid.dot_width() / 2;
        let center_y = grid.dot_height() / 2;

        // Draw floppy disk icon
        let disk_size = 20;
        let left = center_x.saturating_sub(disk_size / 2);
        let top = center_y.saturating_sub(disk_size / 2);
        let right = (center_x + disk_size / 2).min(grid.dot_width() - 1);
        let bottom = (center_y + disk_size / 2).min(grid.dot_height() - 1);

        // Disk outline
        for x in left..=right {
            if top < grid.dot_height() {
                grid.set_dot_with_color(x, top, Color::new(100, 100, 255));
            }
            if bottom < grid.dot_height() {
                grid.set_dot_with_color(x, bottom, Color::new(100, 100, 255));
            }
        }

        for y in top..=bottom {
            if y < grid.dot_height() {
                grid.set_dot_with_color(left, y, Color::new(100, 100, 255));
                if right < grid.dot_width() {
                    grid.set_dot_with_color(right, y, Color::new(100, 100, 255));
                }
            }
        }

        // Metal shutter (top part of disk)
        let shutter_height = disk_size / 3;
        for y in top..(top + shutter_height).min(grid.dot_height()) {
            for x in (left + 2)..=(right.saturating_sub(2)) {
                grid.set_dot_with_color(x, y, Color::new(150, 150, 150));
            }
        }

        // Label area (middle)
        let label_y = center_y;
        if label_y < grid.dot_height() {
            for x in (left + 4)..=(right.saturating_sub(4)) {
                grid.set_dot_with_color(x, label_y, Color::WHITE);
            }
        }

        // Progress bar (Phase 2)
        if matches!(self.phase, Phase::ProgressBar | Phase::Checkmark | Phase::Done) {
            let bar_y = bottom + 4;
            let bar_width = right - left;
            let bar_left = left;

            // Progress based on elapsed time within phase
            let phase_progress = if matches!(self.phase, Phase::ProgressBar) {
                ((self.elapsed.as_secs_f32() - self.duration.as_secs_f32() * 0.3)
                    / (self.duration.as_secs_f32() * 0.4))
                    .min(1.0)
                    .max(0.0)
            } else {
                1.0
            };

            let filled_width = (bar_width as f32 * phase_progress) as usize;

            for x in bar_left..(bar_left + filled_width).min(grid.dot_width()) {
                if bar_y < grid.dot_height() {
                    grid.set_dot_with_color(x, bar_y, Color::GREEN);
                }
            }

            // Bar outline
            for x in bar_left..=(bar_left + bar_width).min(grid.dot_width() - 1) {
                if bar_y.saturating_sub(1) < grid.dot_height() {
                    grid.set_dot_with_color(x, bar_y.saturating_sub(1), Color::WHITE);
                }
                if bar_y + 1 < grid.dot_height() {
                    grid.set_dot_with_color(x, bar_y + 1, Color::WHITE);
                }
            }
        }

        // Checkmark (Phase 3)
        if matches!(self.phase, Phase::Checkmark | Phase::Done) {
            let check_x = right + 8;
            let check_y = center_y;

            // Draw checkmark
            let check_points = vec![
                (check_x, check_y),
                (check_x + 1, check_y + 1),
                (check_x + 2, check_y + 2),
                (check_x + 3, check_y + 1),
                (check_x + 4, check_y),
                (check_x + 5, check_y.saturating_sub(1)),
                (check_x + 6, check_y.saturating_sub(2)),
            ];

            for (x, y) in check_points {
                if x < grid.dot_width() && y < grid.dot_height() {
                    grid.set_dot_with_color(x, y, Color::GREEN);
                }
            }
        }
    }

    fn name(&self) -> &str {
        "Save"
    }

    fn duration(&self) -> Option<Duration> {
        Some(self.duration)
    }
}
