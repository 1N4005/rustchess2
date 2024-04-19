mod tests {
    #[test]
    fn test_perft() {
        let mut board1: rustchess2::game::Board = rustchess2::game::BoardBuilder::new()
            .set_position(rustchess2::game::STARTPOS.to_owned())
            .build();
        let mut board2: rustchess2::game::Board = rustchess2::game::BoardBuilder::new()
            .set_position(rustchess2::game::KIWIPETE.to_owned())
            .build();

        println!("startpos: depth 1");
        let mut positions = movegen::perft::perft(&mut board1, 1, 0);
        assert_eq!(positions, 20);

        println!("startpos: depth 2");
        positions = movegen::perft::perft(&mut board1, 2, 0);
        assert_eq!(positions, 400);

        println!("startpos: depth 3");
        positions = movegen::perft::perft(&mut board1, 3, 0);
        assert_eq!(positions, 8902);

        println!("startpos: depth 4");
        positions = movegen::perft::perft(&mut board1, 4, 0);
        assert_eq!(positions, 197281);

        println!("startpos: depth 5");
        positions = movegen::perft::perft(&mut board1, 5, 0);
        assert_eq!(positions, 4865609);

        println!("kiwipete: depth 1");
        positions = movegen::perft::perft(&mut board2, 1, 0);
        assert_eq!(positions, 48);

        println!("kiwipete: depth 2");
        positions = movegen::perft::perft(&mut board2, 2, 0);
        assert_eq!(positions, 2039);

        println!("kiwipete: depth 3");
        positions = movegen::perft::perft(&mut board2, 3, 0);
        assert_eq!(positions, 97862);

        println!("kiwipete: depth 4");
        positions = movegen::perft::perft(&mut board2, 4, 0);
        assert_eq!(positions, 4085603);
    }
}
