pub enum GameEvent {
    Cursor(f64, f64),
    SelectStart,
    SelectEnd,
    ReadyToPlay,
    SquadSpawn,
    SquadMove,
    Modifier1Start,
    Modifier1End,
    Modifier2Start,
    Modifier2End,
    ZoomIn,
    ZoomOut,
    Resize(f64, f64)
}