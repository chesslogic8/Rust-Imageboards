#[derive(Clone)]
pub struct BoardDef {
    pub slug: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
}

// Add/remove boards here!
pub const BOARDS: &[BoardDef] = &[
    BoardDef {
        slug: "chess",
        name: "General Chess",
        desc: "Discuss chess questions, general strategy, analysis, news, and all things chess. Friendly for all levels.",
    },
    BoardDef {
        slug: "puzzles",
        name: "Puzzles",
        desc: "Share, solve, and discuss chess puzzles and studies. Tactics, checkmates, and brilliant finds!",
    },
    BoardDef {
        slug: "openings",
        name: "Openings",
        desc: "Debate opening theory, share repertoires, and explore move orders from the first move onward.",
    },
    // Example of adding another board:
    // BoardDef {
    //     slug: "endgames",
    //     name: "Endgames",
    //     desc: "All about chess endgames: techniques, studies, and tricky endings.",
    // },
];