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
        "capture" => KwCapture,
        "synthesize" => KwSynthesize,
        "chain" => KwChain,
        "to" => KwTo,
        "orbite" => KwOrbite,
        // declaración
        "atom" => KwAtom,
        "molecule" => KwMolecule,
        "reaction" => KwReaction,
        // tipos
        "symbol" => KwSymbol,
        "atom_num" => KwAtomNum,
        "mass" => KwMass,
        "polarized" => KwPolarized,
        "pos" => KwTrue,
        "neg" => KwFalse,
        "voidstate" => KwVoidState, // case-insensitive
        "formula" => KwFormula,
        "ion" => KwIon,
        "solution" => KwSolution,
        "sample" => KwSample,
        // lógicos
        "and" => And,
        "or" => Or,
        "not" => Not,
        _ => return None,
    })
}
