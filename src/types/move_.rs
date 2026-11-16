//! Encoded moves and a fixed-capacity move list.

use std::fmt;

use super::{PieceKind, Square};

/// A pseudo-legal move encoded in 16 bits.
///
/// Layout:
/// - bits 0..5: from square
/// - bits 6..11: to square
/// - bits 12..15: move flag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move(u16);

impl Move {
    const NULL: Move = Move(0);

    /// Encode a move.
    pub const fn new(from: Square, to: Square, flag: MoveFlag) -> Move {
        Move((from.index() as u16) | ((to.index() as u16) << 6) | ((flag as u16) << 12))
    }

    /// Source square.
    pub const fn from_square(self) -> Square {
        Square::from_index((self.0 & 0x3f) as u8)
    }

    /// Destination square.
    pub const fn to_square(self) -> Square {
        Square::from_index(((self.0 >> 6) & 0x3f) as u8)
    }

    /// Move flag.
    pub fn flag(self) -> MoveFlag {
        MoveFlag::from_u16(self.0 >> 12).expect("valid move flag")
    }

    /// True if the move captures something.
    pub fn is_capture(self) -> bool {
        self.flag().is_capture()
    }

    /// True if the move is a promotion.
    pub fn is_promotion(self) -> bool {
        self.flag().is_promotion()
    }

    /// Promoted piece kind, if this is a promotion.
    pub fn promotion_kind(self) -> Option<PieceKind> {
        self.flag().promotion_kind()
    }

    /// True if the move is castling.
    pub fn is_castling(self) -> bool {
        self.flag().is_castling()
    }

    /// True if the move is en passant.
    pub fn is_en_passant(self) -> bool {
        self.flag().is_en_passant()
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from_square(), self.to_square())?;
        if let Some(kind) = self.promotion_kind() {
            write!(f, "{}", kind.as_char())?;
        }
        Ok(())
    }
}

/// Extra information encoded in a [`Move`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum MoveFlag {
    /// Normal non-capture move.
    Quiet = 0,
    /// Pawn double push.
    DoublePush = 1,
    /// Kingside castling.
    KingCastle = 2,
    /// Queenside castling.
    QueenCastle = 3,
    /// Normal capture
    Capture = 4,
    /// En passant capture.
    EpCapture = 5,
    /// Promotion to knight.
    PromoN = 8,
    /// Promotion to bishop.
    PromoB = 9,
    /// Promotion to rook.
    PromoR = 10,
    /// Promotion to queen.
    PromoQ = 11,
    /// Capture promotion to knight.
    PromoCaptureN = 12,
    /// Capture promotion to bishop.
    PromoCaptureB = 13,
    /// Capture promotion to rook.
    PromoCaptureR = 14,
    /// Capture promotion to queen.
    PromoCaptureQ = 15,
}

impl MoveFlag {
    /// Decode flag bits.
    pub const fn from_u16(bits: u16) -> Option<MoveFlag> {
        match bits {
            0 => Some(MoveFlag::Quiet),
            1 => Some(MoveFlag::DoublePush),
            2 => Some(MoveFlag::KingCastle),
            3 => Some(MoveFlag::QueenCastle),
            4 => Some(MoveFlag::Capture),
            5 => Some(MoveFlag::EpCapture),
            8 => Some(MoveFlag::PromoN),
            9 => Some(MoveFlag::PromoB),
            10 => Some(MoveFlag::PromoR),
            11 => Some(MoveFlag::PromoQ),
            12 => Some(MoveFlag::PromoCaptureN),
            13 => Some(MoveFlag::PromoCaptureB),
            14 => Some(MoveFlag::PromoCaptureR),
            15 => Some(MoveFlag::PromoCaptureQ),
            _ => None,
        }
    }

    /// True if this flag captures.
    pub const fn is_capture(self) -> bool {
        matches!(
            self,
            MoveFlag::Capture
                | MoveFlag::EpCapture
                | MoveFlag::PromoCaptureN
                | MoveFlag::PromoCaptureB
                | MoveFlag::PromoCaptureR
                | MoveFlag::PromoCaptureQ
        )
    }

    /// True if this flag promtes.
    pub const fn is_promotion(self) -> bool {
        matches!(
            self,
            MoveFlag::PromoN
                | MoveFlag::PromoB
                | MoveFlag::PromoR
                | MoveFlag::PromoQ
                | MoveFlag::PromoCaptureN
                | MoveFlag::PromoCaptureB
                | MoveFlag::PromoCaptureR
                | MoveFlag::PromoCaptureQ
        )
    }

    /// Promotion piece kind, if any.
    pub const fn promotion_kind(self) -> Option<PieceKind> {
        match self {
            MoveFlag::PromoN | MoveFlag::PromoCaptureN => Some(PieceKind::Knight),
            MoveFlag::PromoB | MoveFlag::PromoCaptureB => Some(PieceKind::Bishop),
            MoveFlag::PromoR | MoveFlag::PromoCaptureR => Some(PieceKind::Rook),
            MoveFlag::PromoQ | MoveFlag::PromoCaptureQ => Some(PieceKind::Queen),
            _ => None,
        }
    }

    /// True if this flag is castling.
    pub const fn is_castling(self) -> bool {
        matches!(self, MoveFlag::KingCastle | MoveFlag::QueenCastle)
    }

    /// True if this flag is en passant.
    pub const fn is_en_passant(self) -> bool {
        matches!(self, MoveFlag::EpCapture)
    }
}

/// Fixed-capacity move list. No heap allocation.
pub struct MoveList {
    moves: [Move; MoveList::CAPACITY],
    len: usize,
}

impl MoveList {
    /// Maximum number of moves stored.
    pub const CAPACITY: usize = 512;

    /// Create an empty list.
    pub const fn new() -> MoveList {
        MoveList {
            moves: [Move::NULL; Self::CAPACITY],
            len: 0,
        }
    }

    /// Append a move. Panics if capacity is exceeded.
    pub fn push(&mut self, mv: Move) {
        assert!(self.len < Self::CAPACITY, "move list overflow");
        self.moves[self.len] = mv;
        self.len += 1;
    }

    /// Number of moves stored.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// True if no moves are stored.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Stored moves as a slice.
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    /// Iterate over stored moves
    pub fn iter(&self) -> std::slice::Iter<'_, Move> {
        self.as_slice().iter()
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sq(s: &str) -> Square {
        Square::from_algebraic(s).unwrap()
    }

    #[test]
    fn move_round_trip() {
        let mv = Move::new(sq("e2"), sq("e4"), MoveFlag::DoublePush);
        assert_eq!(mv.from_square(), sq("e2"));
        assert_eq!(mv.to_square(), sq("e4"));
        assert_eq!(mv.flag(), MoveFlag::DoublePush);
        assert_eq!(mv.to_string(), "e2e4");
    }

    #[test]
    fn promotion_display() {
        let mv = Move::new(sq("a7"), sq("a8"), MoveFlag::PromoQ);
        assert!(mv.is_promotion());
        assert_eq!(mv.promotion_kind(), Some(PieceKind::Queen));
        assert_eq!(mv.to_string(), "a7a8q");
    }

    #[test]
    fn capture_flags() {
        assert!(MoveFlag::Capture.is_capture());
        assert!(MoveFlag::EpCapture.is_capture());
        assert!(MoveFlag::PromoCaptureN.is_capture());
        assert!(!MoveFlag::Quiet.is_capture());
        assert!(!MoveFlag::PromoN.is_capture());
    }

    #[test]
    fn move_list_push() {
        let mut list = MoveList::new();
        assert!(list.is_empty());
        list.push(Move::new(sq("e2"), sq("e4"), MoveFlag::DoublePush));
        assert_eq!(list.len(), 1);
        assert_eq!(list.as_slice()[0].to_string(), "e2e4");
    }
}
