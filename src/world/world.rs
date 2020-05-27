pub struct World {
    map: super::map::Map,
    players: Vec<super::player::Player>,
}

impl World {
    pub fn new() -> World {
        let mut generator = super::generator::Generator::new();

        generator.add_tile(0, 2f32);
        generator.add_tile(1, 4f32);
        generator.add_tile(2, 4f32);
        generator.add_tile(3, 1f32);

        let mut constraints = std::collections::HashMap::new();

        constraints.insert(0, vec![0, 3]);
        constraints.insert(1, vec![1, 2, 3]);
        constraints.insert(2, vec![1, 2, 3]);
        constraints.insert(3, vec![0, 1, 2, 3]);

        let mut initials: Vec<Vec<bool>> = Vec::new();

        for _ in 0..40 * 30 {
            initials.push(vec![true; 4]);
        }

        for index in 0..40 * 30 {
            let x = index % 40;
            let y = index / 40;

            if x == 0 || x == 39 || y == 0 || y == 29 {
                initials[index][0] = false;
                initials[index][1] = false;
                initials[index][2] = false;
                initials[index][3] = true;
            }
        }

        initials[1 + 1 * 40][0] = true;
        initials[1 + 1 * 40][1] = false;
        initials[1 + 1 * 40][2] = false;
        initials[1 + 1 * 40][3] = false;

        initials[38 + 28 * 40][0] = true;
        initials[38 + 28 * 40][1] = false;
        initials[38 + 28 * 40][2] = false;
        initials[38 + 28 * 40][3] = false;

        World {
            map: super::map::Map::from(
                40,
                30,
                generator
                    .generate(40, 30, &constraints, Some(initials))
                    .unwrap(),
            ),
            players: Vec::new(),
        }
    }

    pub fn map(&self) -> &super::map::Map {
        &self.map
    }

    pub fn players(&self) -> &Vec<super::player::Player> {
        &self.players
    }

    pub fn add_player(&mut self, id: u64) {
        self.players.push(super::player::Player::new(id, 1, 1));
    }

    pub fn remove_player(&mut self, id: u64) {
        match self.players.iter().position(|player| id == player.id()) {
            Some(index) => {
                self.players.remove(index);
            }
            None => {}
        };
    }
}
