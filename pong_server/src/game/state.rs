use crate::protocol::packet::GameStatePayload;
pub const COURT_WIDTH: f32 = 800.0;
pub const COURT_HEIGHT: f32 = 600.0;
pub const PADDLE_HEIGHT: f32 = 90.0;
pub const PADDLE_WIDTH: f32 = 15.0;
pub const PADDLE_SPEED: f32 = 7.0;
pub const BALL_SIZE: f32 = 12.0;
pub const WINNING_SCORE: u8 = 5;
const MAX_BALL_SPEED: f32 = 12.0;

pub struct GameState {
    pub paddle1_y: f32,
    pub paddle2_y: f32,
    pub ball_x: f32,
    pub ball_y: f32,
    pub ball_vx: f32,
    pub ball_vy: f32,
    pub score_p1: u8,
    pub score_p2: u8,
}
impl GameState {
    pub fn new() -> Self {
        Self {
            paddle1_y: 255.0,
            paddle2_y: 255.0,
            ball_x: 394.0,
            ball_y: 294.0,
            ball_vx: 5.0,
            ball_vy: 3.0,
            score_p1: 0,
            score_p2: 0,
        }
    }
    pub fn move_paddle(&mut self, role: u8, direction: i8) {
        let paddle = match role {
            1 => &mut self.paddle1_y,
            2 => &mut self.paddle2_y,
            _ => return,
        };
        *paddle = (*paddle + direction.signum() as f32 * PADDLE_SPEED)
            .clamp(0.0, COURT_HEIGHT - PADDLE_HEIGHT);
    }
    pub fn update_physics(&mut self) -> Option<u8> {
        let old_x = self.ball_x;
        self.ball_x += self.ball_vx;
        self.ball_y += self.ball_vy;
        if self.ball_y < 0.0 {
            self.ball_y = -self.ball_y;
            self.ball_vy = self.ball_vy.abs();
        } else if self.ball_y > COURT_HEIGHT - BALL_SIZE {
            self.ball_y = 2.0 * (COURT_HEIGHT - BALL_SIZE) - self.ball_y;
            self.ball_vy = -self.ball_vy.abs();
        }
        let left_face = 30.0 + PADDLE_WIDTH;
        let right_face = COURT_WIDTH - 30.0 - PADDLE_WIDTH;
        // Swept crossing prevents tunnelling through the paddle at high speed.
        if self.ball_vx < 0.0
            && old_x >= left_face
            && self.ball_x <= left_face
            && self.ball_y + BALL_SIZE >= self.paddle1_y
            && self.ball_y <= self.paddle1_y + PADDLE_HEIGHT
        {
            self.ball_x = left_face;
            self.bounce(self.paddle1_y, 1.0);
        }
        if self.ball_vx > 0.0
            && old_x + BALL_SIZE <= right_face
            && self.ball_x + BALL_SIZE >= right_face
            && self.ball_y + BALL_SIZE >= self.paddle2_y
            && self.ball_y <= self.paddle2_y + PADDLE_HEIGHT
        {
            self.ball_x = right_face - BALL_SIZE;
            self.bounce(self.paddle2_y, -1.0);
        }
        if self.ball_x + BALL_SIZE <= 0.0 {
            self.score_p2 += 1;
            self.reset_ball(1.0);
        } else if self.ball_x >= COURT_WIDTH {
            self.score_p1 += 1;
            self.reset_ball(-1.0);
        }
        if self.score_p1 >= WINNING_SCORE {
            Some(1)
        } else if self.score_p2 >= WINNING_SCORE {
            Some(2)
        } else {
            None
        }
    }
    fn bounce(&mut self, paddle_y: f32, direction: f32) {
        self.ball_vx = (self.ball_vx.abs() * 1.05).min(MAX_BALL_SPEED) * direction;
        let offset = (self.ball_y + BALL_SIZE / 2.0 - paddle_y - PADDLE_HEIGHT / 2.0)
            / (PADDLE_HEIGHT / 2.0);
        self.ball_vy = offset.clamp(-1.0, 1.0) * 5.0;
    }
    fn reset_ball(&mut self, direction: f32) {
        self.ball_x = (COURT_WIDTH - BALL_SIZE) / 2.0;
        self.ball_y = (COURT_HEIGHT - BALL_SIZE) / 2.0;
        self.ball_vx = 5.0 * direction;
        self.ball_vy = if (self.score_p1 + self.score_p2).is_multiple_of(2) {
            3.0
        } else {
            -3.0
        };
    }
    pub fn to_payload(&self) -> GameStatePayload {
        GameStatePayload {
            paddle1_y: self.paddle1_y.round() as u16,
            paddle2_y: self.paddle2_y.round() as u16,
            ball_x: self.ball_x.max(0.0).round() as u16,
            ball_y: self.ball_y.round() as u16,
            score_p1: self.score_p1,
            score_p2: self.score_p2,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paddles_stay_in_court_and_still_means_still() {
        let mut g = GameState::new();
        for _ in 0..200 {
            g.move_paddle(1, -1);
            g.move_paddle(2, 1);
        }
        assert_eq!((g.paddle1_y, g.paddle2_y), (0.0, 510.0));
        g.move_paddle(1, 0);
        g.move_paddle(9, 1);
        assert_eq!((g.paddle1_y, g.paddle2_y), (0.0, 510.0));
    }
    #[test]
    fn swept_paddle_collision_and_speed_cap() {
        let mut g = GameState::new();
        g.ball_x = 50.0;
        g.ball_y = 290.0;
        g.ball_vx = -30.0;
        g.ball_vy = 0.0;
        g.update_physics();
        assert_eq!(g.ball_x, 45.0);
        assert_eq!(g.ball_vx, 12.0);
        assert_eq!(g.score_p2, 0);
    }
    #[test]
    fn wall_reflection_stays_inside() {
        let mut g = GameState::new();
        g.ball_y = 1.0;
        g.ball_vy = -3.0;
        g.update_physics();
        assert_eq!(g.ball_y, 2.0);
        assert!(g.ball_vy > 0.0);
    }
    #[test]
    fn goals_reset_ball_and_five_points_win() {
        let mut g = GameState::new();
        for point in 1..=5 {
            g.ball_x = 799.0;
            g.ball_y = 0.0;
            g.ball_vx = 5.0;
            assert_eq!(g.update_physics(), if point == 5 { Some(1) } else { None });
            assert_eq!(g.score_p1, point);
            assert_eq!(g.ball_x, 394.0);
        }
    }
}
