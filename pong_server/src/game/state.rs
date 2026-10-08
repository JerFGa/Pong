use crate::protocol::packet::GameStatePayload;

pub const COURT_WIDTH: f32 = 800.0;
pub const COURT_HEIGHT: f32 = 600.0;
pub const PADDLE_HEIGHT: f32 = 90.0;
pub const PADDLE_WIDTH: f32 = 15.0;
pub const PADDLE_SPEED: f32 = 7.0;
pub const BALL_SIZE: f32 = 12.0;
pub const WINNING_SCORE: u8 = 5;

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
            paddle1_y: (COURT_HEIGHT - PADDLE_HEIGHT) / 2.0,
            paddle2_y: (COURT_HEIGHT - PADDLE_HEIGHT) / 2.0,
            ball_x: COURT_WIDTH / 2.0,
            ball_y: COURT_HEIGHT / 2.0,
            ball_vx: 5.0,
            ball_vy: 3.0,
            score_p1: 0,
            score_p2: 0,
        }
    }

    pub fn move_paddle(&mut self, player_role: u8, direction: i8) {
        let paddle = if player_role == 1 {
            &mut self.paddle1_y
        } else {
            &mut self.paddle2_y
        };

        if direction < 0 {
            *paddle = (*paddle - PADDLE_SPEED).max(0.0);
        } else if direction > 0 {
            *paddle = (*paddle + PADDLE_SPEED).min(COURT_HEIGHT - PADDLE_HEIGHT);
        }
    }

    pub fn update_physics(&mut self) -> Option<u8> {
        self.ball_x += self.ball_vx;
        self.ball_y += self.ball_vy;

        // Rebote en paredes superior e inferior
        if self.ball_y <= 0.0 {
            self.ball_y = 0.0;
            self.ball_vy = -self.ball_vy;
        } else if self.ball_y >= COURT_HEIGHT - BALL_SIZE {
            self.ball_y = COURT_HEIGHT - BALL_SIZE;
            self.ball_vy = -self.ball_vy;
        }

        // Colisión con Paleta 1 (Izquierda)
        let p1_x = 30.0;
        if self.ball_x <= p1_x + PADDLE_WIDTH
            && self.ball_x >= p1_x
            && self.ball_y + BALL_SIZE >= self.paddle1_y
            && self.ball_y <= self.paddle1_y + PADDLE_HEIGHT
            && self.ball_vx < 0.0
        {
            self.ball_vx = -self.ball_vx * 1.05; // Leve incremento de velocidad
        }

        // Colisión con Paleta 2 (Derecha)
        let p2_x = COURT_WIDTH - 30.0 - PADDLE_WIDTH;
        if self.ball_x + BALL_SIZE >= p2_x
            && self.ball_x <= p2_x + PADDLE_WIDTH
            && self.ball_y + BALL_SIZE >= self.paddle2_y
            && self.ball_y <= self.paddle2_y + PADDLE_HEIGHT
            && self.ball_vx > 0.0
        {
            self.ball_vx = -self.ball_vx * 1.05;
        }

        // Punto para Jugador 2 (Pasadizo izquierdo)
        if self.ball_x <= 0.0 {
            self.score_p2 += 1;
            self.reset_ball(1.0);
            if self.score_p2 >= WINNING_SCORE {
                return Some(2); // Ganó P2
            }
        }

        // Punto para Jugador 1 (Pasadizo derecho)
        if self.ball_x >= COURT_WIDTH {
            self.score_p1 += 1;
            self.reset_ball(-1.0);
            if self.score_p1 >= WINNING_SCORE {
                return Some(1); // Ganó P1
            }
        }

        None
    }

    fn reset_ball(&mut self, direction_x: f32) {
        self.ball_x = COURT_WIDTH / 2.0;
        self.ball_y = COURT_HEIGHT / 2.0;
        self.ball_vx = 5.0 * direction_x;
        self.ball_vy = 3.0;
    }

    pub fn to_payload(&self) -> GameStatePayload {
        GameStatePayload {
            paddle1_y: self.paddle1_y.round() as u16,
            paddle2_y: self.paddle2_y.round() as u16,
            ball_x: self.ball_x.round() as u16,
            ball_y: self.ball_y.round() as u16,
            score_p1: self.score_p1,
            score_p2: self.score_p2,
        }
    }
}
