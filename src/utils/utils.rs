use crate::utils::enums::TokenKind;

pub fn keyword_kind(s_lower: &str) -> Option<TokenKind> {
    use TokenKind::*;
    Some(match s_lower {
        // control
        "itest" => KwITest,
        "notest" => KwNotest,
        "inotest" => KwInotest,
        "inhibit" => KwInhibit,
        "release" => KwRelease,
        "emitln" => KwEmitln,
        "emit" => KwEmit,
        "synthesize" => KwSynthesize,
        "chain" => KwChain,
        "to" => KwTo,
        // declaración
        "atom" => KwAtom,
        "molecule" => KwMolecule,
        "reaction" => KwReaction,
        // tipos
        "symbol" => KwSymbol,
        "atom_num" => KwAtomNum,
        "mass" => KwMass,
        "polarized" => KwPolarized,
        "voidstate" => KwVoidState, // case-insensitive
        "formula" => KwFormula,
        "ion" => KwIon,
        "solution" => KwSolution,
        // lógicos
        "and" => And,
        "or" => Or,
        "not" => Not,
        _ => return None,
    })
}