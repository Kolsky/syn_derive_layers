#[cfg(test)]
mod tests {
    use derive_macro::*;
    use trait_layers::Layers as _;

    macro_rules! assert_matches {
        ($expression:expr, $( $pattern:pat )|+ $( if $guard: expr )? $(,)?) => {
            assert!(matches!($expression, $($pattern)|+ $( if $guard )?))
        };
    }

    #[derive(Root, Layers)]
    enum MainLayers {
        Background(Background),
        Level(Level),
        Char,
        Foreground(Void),
        Ui(Ui),
    }


    #[derive(Layers)]
    enum Background {
        Static,
        DynamicBack,
        DynamicFront,
    }

    #[derive(Layers)]
    enum Level {
        Walls,
        Tiles,
    }

    #[derive(Layers)]
    enum Void {}

    #[derive(Layers)]
    enum Ui {
        Back,
        Canvas(OnCanvas),
    }

    #[derive(Layers)]
    enum OnCanvas {
        Rectangles,
        Triangles(Void),
        Buttons,
    }

    #[test]
    fn it_is_ordered() {
        assert_eq!(MainLayers::Background(Background::Static).as_num(), 0);
        assert_eq!(MainLayers::Background(Background::DynamicBack).as_num(), 1);
        assert_eq!(MainLayers::Background(Background::DynamicFront).as_num(), 2);
        assert_eq!(MainLayers::Level(Level::Walls).as_num(), 3);
        assert_eq!(MainLayers::Level(Level::Tiles).as_num(), 4);
        assert_eq!(MainLayers::Char.as_num(), 5);
        assert_eq!(MainLayers::Ui(Ui::Back).as_num(), 6);
        assert_eq!(MainLayers::Ui(Ui::Canvas(OnCanvas::Rectangles)).as_num(), 7);
        assert_eq!(MainLayers::Ui(Ui::Canvas(OnCanvas::Buttons)).as_num(), 8);
        assert_eq!(Level::Walls.as_num(), 0);
    }

    #[test]
    fn it_is_symmetric() {
        assert_matches!(MainLayers::try_from_num(0), Some(MainLayers::Background(Background::Static)));
        assert_matches!(MainLayers::try_from_num(1), Some(MainLayers::Background(Background::DynamicBack)));
        assert_matches!(MainLayers::try_from_num(2), Some(MainLayers::Background(Background::DynamicFront)));
        assert_matches!(MainLayers::try_from_num(3), Some(MainLayers::Level(Level::Walls)));
        assert_matches!(MainLayers::try_from_num(4), Some(MainLayers::Level(Level::Tiles)));
        assert_matches!(MainLayers::try_from_num(5), Some(MainLayers::Char));
        assert_matches!(MainLayers::try_from_num(6), Some(MainLayers::Ui(Ui::Back)));
        assert_matches!(MainLayers::try_from_num(7), Some(MainLayers::Ui(Ui::Canvas(OnCanvas::Rectangles))));
        assert_matches!(MainLayers::try_from_num(8), Some(MainLayers::Ui(Ui::Canvas(OnCanvas::Buttons))));
        assert_matches!(MainLayers::try_from_num(9), None);
    }

    #[test]
    fn it_has_expected_count() {
        assert_eq!(MainLayers::COUNT, 9);
        assert_eq!(Background::COUNT, 3);
        assert_eq!(Level::COUNT, 2);
        assert_eq!(Void::COUNT, 0);
        assert_eq!(Ui::COUNT, 3);
        assert_eq!(OnCanvas::COUNT, 2);
    }
}
