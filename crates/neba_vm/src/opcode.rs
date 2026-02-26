/// Instruction set della Neba VM (v0.2.0).
///
/// Ogni istruzione è codificata come 1 byte (opcode) seguito da 0-3 byte
/// di operandi. I formati usati:
///   - [u8]   → 1 byte operando
///   - [u16]  → 2 byte little-endian
///   - [i16]  → 2 byte little-endian signed (offset di salto)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    // ── Costanti ──────────────────────────────────────────────────────────
    /// `Const [u16]` — push constants[idx]
    Const,
    /// push true
    True,
    /// push false
    False,
    /// push None
    Nil,

    // ── Stack ─────────────────────────────────────────────────────────────
    /// Scarta il top
    Pop,
    /// Duplica il top
    Dup,
    /// Scarta N elementi dal top (per cleanup di blocchi)  [u8]
    PopN,

    // ── Variabili locali ──────────────────────────────────────────────────
    /// `LoadLocal [u8]` — push frame.locals[idx]
    LoadLocal,
    /// `StoreLocal [u8]` — frame.locals[idx] = top (non poppare)
    StoreLocal,

    // ── Upvalue (closure) ─────────────────────────────────────────────────
    /// `LoadUpval [u8]`
    LoadUpval,
    /// `StoreUpval [u8]`
    StoreUpval,

    // ── Variabili globali ─────────────────────────────────────────────────
    /// `LoadGlobal [u16]` — push globals[names[idx]]
    LoadGlobal,
    /// `StoreGlobal [u16]` — globals[names[idx]] = pop()
    StoreGlobal,
    /// `DefGlobal [u16] [u8:mutable]` — definisce nuova variabile globale
    DefGlobal,

    // ── Aritmetica ────────────────────────────────────────────────────────
    Add, Sub, Mul, Div, IntDiv, Mod, Pow,
    /// Negazione unaria
    Neg,

    // ── Bitwise ───────────────────────────────────────────────────────────
    BitAnd, BitOr, BitXor, BitNot, Shl, Shr,

    // ── Confronto ─────────────────────────────────────────────────────────
    Eq, Ne, Lt, Le, Gt, Ge,

    // ── Logica ────────────────────────────────────────────────────────────
    /// Negazione booleana
    Not,

    // ── Salti ─────────────────────────────────────────────────────────────
    /// `Jump [i16]` — salto incondizionato (relativo all'istruzione dopo)
    Jump,
    /// `JumpFalse [i16]` — salto se top è falsy, POP
    JumpFalse,
    /// `JumpTrue [i16]` — salto se top è truthy, POP
    JumpTrue,
    /// `JumpFalsePeek [i16]` — salto se top è falsy, NON POP (and/or short-circuit)
    JumpFalsePeek,
    /// `JumpTruePeek [i16]` — salto se top è truthy, NON POP
    JumpTruePeek,

    // ── Funzioni ──────────────────────────────────────────────────────────
    /// `MakeClosure [u16]` — crea Closure da fn_protos[idx]
    MakeClosure,
    /// `Call [u8:argc]` — chiama top-of-stack-after-args con argc argomenti
    Call,
    /// `CallMethod [u16:name_idx] [u8:argc]` — chiama obj.method(args) passando self
//    CallMethod,
    // ── Funzioni ──────────────────────────────────────────────────────────
    /// `MakeClosure [u16]` — crea Closure da fn_protos[idx]
//    MakeClosure,
    /// `Call [u8:argc]` — chiama top-of-stack-after-args con argc argomenti
    // Call,
    /// `CallMethod [u16:name_idx] [u8:argc]` — chiama obj.method(args) passando self
    CallMethod,
    /// `Return` — ritorna il top dello stack al chiamante
    Return,
    /// `ReturnNil` — ritorna None al chiamante
    ReturnNil,
    // ── Collezioni ────────────────────────────────────────────────────────
    /// `MakeArray [u16:count]` — pop count items (ordine LIFO), push Array
    MakeArray,
    /// `GetIndex` — pop idx, pop obj, push obj[idx]
    GetIndex,
    /// `SetIndex` — pop val, pop idx, pop obj, obj[idx]=val, push None
    SetIndex,
    /// `MakeRange [u8:inclusive]` — pop end, pop start, push Range/Array
    MakeRange,

    // ── Classi / istanze ──────────────────────────────────────────────────
    /// `GetField [u16:name_idx]` — pop obj, push obj.field
    GetField,
    /// `SetField [u16:name_idx]` — pop val, pop obj, obj.field=val, push None
    SetField,
    /// `MakeInstance [u16:class_name_idx]` — crea istanza con campi default
    MakeInstance,

    // ── Option / Result ───────────────────────────────────────────────────
    /// `MakeSome` — pop v, push Some(v)
    MakeSome,
    /// `MakeOk` — pop v, push Ok(v)
    MakeOk,
    /// `MakeErr` — pop v, push Err(v)
    MakeErr,

    // ── Membership ────────────────────────────────────────────────────────
    In,
    NotIn,
    Is,

    // ── Pattern matching helpers ───────────────────────────────────────────
    /// `IsSome [i16]` — se top NON è Some, jump (peek, non pop)
    IsSome,
    /// `IsNone [i16]` — se top NON è None, jump
    IsNone,
    /// `IsOk [i16]`
    IsOk,
    /// `IsErr [i16]`
    IsErr,
    /// `Unwrap` — pop Some/Ok/Err(v), push v
    Unwrap,
    /// `MatchLit [u16:const_idx] [i16]` — se top != const, jump (peek)
    MatchLit,
    /// `MatchRange [u16:lo] [u16:hi] [u8:incl] [i16]` — controlla range
    MatchRange,

    // ── Iterazione ────────────────────────────────────────────────────────
    /// Converte top in iterabile (già un Array o Range → Array)
    IntoIter,
    /// `IterNext [u8:iter_local] [u8:var_local] [i16:jump_done]`
    /// Avanza l'iteratore; se esaurito salta. Altrimenti scrive var_local.
    IterNext,

    // ── F-string ──────────────────────────────────────────────────────────
    /// `BuildStr [u16:n]` — pop n strings, concatena, push risultato
    BuildStr,
    /// Converte top in Str (come str())
    ToStr,

    // ── Misc ──────────────────────────────────────────────────────────────
    Nop,
    Halt,
}

impl Op {
    /// Numero di byte di operandi che seguono l'opcode.
    pub fn operand_bytes(self) -> usize {
        match self {
            Op::Const       => 2,
            Op::PopN        => 1,
            Op::LoadLocal   => 1,
            Op::StoreLocal  => 1,
            Op::LoadUpval   => 1,
            Op::StoreUpval  => 1,
            Op::LoadGlobal  => 2,
            Op::StoreGlobal => 2,
            Op::DefGlobal   => 3,   // [u16 name] [u8 mutable]
            Op::Jump        => 2,
            Op::JumpFalse   => 2,
            Op::JumpTrue    => 2,
            Op::JumpFalsePeek => 2,
            Op::JumpTruePeek  => 2,
            Op::MakeClosure => 2,
            Op::Call        => 1,
            Op::CallMethod  => 3,   // [u16 name] [u8 argc]
            Op::MakeArray   => 2,
            Op::MakeRange   => 1,
            Op::GetField    => 2,
            Op::SetField    => 2,
            Op::MakeInstance => 2,
            Op::IsSome      => 2,
            Op::IsNone      => 2,
            Op::IsOk        => 2,
            Op::IsErr       => 2,
            Op::MatchLit    => 4,   // [u16 const] [i16 offset]
            Op::MatchRange  => 7,   // [u16 lo] [u16 hi] [u8 incl] [i16 offset]
            Op::IterNext    => 4,   // [u8 iter_local] [u8 var_local] [i16 jump]
            Op::BuildStr    => 2,
            _               => 0,
        }
    }

    pub fn from_u8(b: u8) -> Option<Self> {
        // SAFETY: tutti i valori 0..=N sono varianti valide
        if b <= Op::Halt as u8 {
            Some(unsafe { std::mem::transmute(b) })
        } else {
            None
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
