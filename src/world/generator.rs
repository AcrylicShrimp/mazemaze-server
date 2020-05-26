extern crate ordered_float;
extern crate rand;
extern crate rand_distr;

use rand::distributions::Distribution;

pub struct Tile {
    id: u8,
    weight: f32,
}

pub struct Generator {
    tiles: Vec<Tile>,
    waves: Vec<Vec<bool>>,
    sum_one: Vec<usize>,
    sum_weight: Vec<f32>,
    sum_weight_log_weight: Vec<f32>,
    entropies: Vec<f32>,
}

impl Generator {
    pub fn new() -> Generator {
        Generator {
            tiles: Vec::new(),
            waves: Vec::new(),
            sum_one: Vec::new(),
            sum_weight: Vec::new(),
            sum_weight_log_weight: Vec::new(),
            entropies: Vec::new(),
        }
    }

    pub fn add_tile(&mut self, id: u8, weight: f32) {
        self.tiles.push(Tile { id, weight });
    }

    pub fn generate(
        &mut self,
        width: usize,
        height: usize,
        constraints: &std::collections::HashMap<usize, Vec<usize>>,
        initial_constraints: Option<Vec<Vec<bool>>>,
    ) -> Option<Vec<u8>> {
        if self.tiles.len() == 0 {
            return None;
        }

        match initial_constraints {
            Some(initial) => {
                self.waves = initial;

                for wave in self.waves.iter() {
                    self.sum_one
                        .push(wave.iter().filter(|&possibility| *possibility).count());
                    self.sum_weight.push(
                        wave.iter()
                            .enumerate()
                            .filter(|&(_, possibility)| *possibility)
                            .map(|(index, _)| self.tiles[index].weight)
                            .sum(),
                    );
                    self.sum_weight_log_weight.push(
                        wave.iter()
                            .enumerate()
                            .filter(|&(_, possibility)| *possibility)
                            .map(|(index, _)| {
                                self.tiles[index].weight * self.tiles[index].weight.log2()
                            })
                            .sum(),
                    );
                    self.entropies.push(
                        self.sum_weight.last().unwrap().log2()
                            - self.sum_weight_log_weight.last().unwrap()
                                / self.sum_weight.last().unwrap(),
                    );
                }
            }
            None => {
                let sum_one = self.tiles.len();
                let sum_weight: f32 = self.tiles.iter().map(|tile| tile.weight).sum();
                let sum_weight_log_weight: f32 = self
                    .tiles
                    .iter()
                    .map(|tile| tile.weight * tile.weight.log2())
                    .sum();
                let entropy = sum_weight.log2() - sum_weight_log_weight / sum_weight;

                for _ in 0..width * height {
                    self.waves.push(vec![true; self.tiles.len()]);
                    self.sum_one.push(sum_one);
                    self.sum_weight.push(sum_weight);
                    self.sum_weight_log_weight.push(sum_weight_log_weight);
                    self.entropies.push(entropy);
                }
            }
        }

        let mut rng = rand::thread_rng();
        let mut min_heap: std::collections::BTreeMap<ordered_float::OrderedFloat<f32>, Vec<usize>> =
            std::collections::BTreeMap::new();

        for index in 0..self.entropies.len() {
            match min_heap.get_mut(&ordered_float::OrderedFloat::from(self.entropies[index])) {
                Some(indices) => indices.push(index),
                None => {
                    min_heap.insert(
                        ordered_float::OrderedFloat::from(self.entropies[index]),
                        vec![index],
                    );
                }
            }
        }

        let mut tiles: Vec<bool> = vec![false; self.tiles.len()];
        let mut stack: Vec<usize> = Vec::new();
        let mut visited_set: std::collections::HashSet<usize> = std::collections::HashSet::new();

        loop {
            let mut entropy: Option<ordered_float::OrderedFloat<f32>> = None;

            {
                if min_heap.len() == 0 {
                    break;
                }

                let minimum = min_heap.iter_mut().next().unwrap();
                let index = *minimum.1.first().unwrap();
                let mut choices = self.waves[index]
                    .iter()
                    .enumerate()
                    .filter(|&(_, possibility)| *possibility)
                    .map(|(index, _)| (self.tiles[index].weight, index))
                    .collect::<Vec<(f32, usize)>>();

                for index in 1..choices.len() {
                    choices[index].0 += choices[index - 1].0;
                }

                let pivot = rand::distributions::Uniform::new(0f32, 1f32).sample(&mut rng)
                    * choices.last().unwrap().0;

                let mut choice = choices.last().unwrap().1;

                for index in 0..choices.len() - 1 {
                    if pivot <= choices[index].0 {
                        choice = choices[index].1;
                        break;
                    }
                }

                for possibility in self.waves[index].iter_mut() {
                    *possibility = false;
                }

                self.waves[index][choice] = true;

                self.sum_one[index] = 0;
                self.sum_weight[index] = 0f32;
                self.sum_weight_log_weight[index] = 0f32;
                self.entropies[index] = f32::NAN;

                minimum.1.remove(
                    minimum
                        .1
                        .iter()
                        .position(|wave_index| *wave_index == index)
                        .unwrap(),
                );

                stack.push(index);

                if minimum.1.len() == 0 {
                    entropy = Some(*minimum.0);
                }
            }

            if entropy.is_some() {
                min_heap.remove(&entropy.unwrap());
            }

            while !stack.is_empty() {
                let index = stack.pop().unwrap();

                if visited_set.contains(&index) {
                    continue;
                }

                visited_set.insert(index);

                for tile in tiles.iter_mut() {
                    *tile = false;
                }

                for tile_index in 0..self.tiles.len() {
                    if !self.waves[index][tile_index] {
                        continue;
                    }

                    for possible_neighbor_tile_index in constraints[&tile_index].iter() {
                        tiles[*possible_neighbor_tile_index] = true;
                    }
                }

                let mut handle_tile = |x: usize, y: usize| {
                    let index = x + y * width;
                    let wave = &mut self.waves[index];
                    let mut entropy = None;

                    for tile_index in 0..self.tiles.len() {
                        if !tiles[tile_index] && wave[tile_index] {
                            entropy = Some(self.entropies[index]);
                            wave[tile_index] = false;
                            self.sum_one[index] -= 1;

                            if self.sum_one[index] == 0 {
                                self.sum_weight[index] = 0f32;
                                self.sum_weight_log_weight[index] = 0f32;
                                self.entropies[index] = f32::NAN;
                            } else {
                                self.sum_weight[index] -= self.tiles[tile_index].weight;
                                self.sum_weight_log_weight[index] -= self.tiles[tile_index].weight
                                    * self.tiles[tile_index].weight.log2();
                                self.entropies[index] = self.sum_weight[index].log2()
                                    - self.sum_weight_log_weight[index] / self.sum_weight[index]
                                    + rand_distr::Normal::new(0f32, 0.1f32)
                                        .unwrap()
                                        .sample(&mut rng);
                            }
                        }
                    }

                    match entropy {
                        Some(entropy) => {
                            let entropy = ordered_float::OrderedFloat::from(entropy);

                            match min_heap.get_mut(&entropy) {
                                Some(indices) => {
                                    indices.remove(
                                        indices
                                            .iter()
                                            .position(|wave_index| *wave_index == index)
                                            .unwrap(),
                                    );

                                    if min_heap[&entropy].len() == 0 {
                                        min_heap.remove(&entropy);
                                    }
                                }
                                None => {}
                            }

                            if self.sum_one[index] != 0 {
                                stack.push(index);
                                match min_heap.get_mut(&ordered_float::OrderedFloat::from(
                                    self.entropies[index],
                                )) {
                                    Some(indices) => {
                                        indices.push(index);
                                    }
                                    None => {
                                        min_heap.insert(
                                            ordered_float::OrderedFloat::from(
                                                self.entropies[index],
                                            ),
                                            vec![index],
                                        );
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                };

                let x = index % width;
                let y = index / width;

                if y != 0 {
                    handle_tile(x, y - 1);
                }

                if y != height - 1 {
                    handle_tile(x, y + 1);
                }

                if x != 0 {
                    handle_tile(x - 1, y);
                }

                if x != width - 1 {
                    handle_tile(x + 1, y);
                }
            }

            visited_set.clear();
        }

        let mut result: Vec<u8> = Vec::with_capacity(self.waves.len());

        for wave in self.waves.iter() {
            for index in 0..wave.len() {
                if wave[index] {
                    result.push(self.tiles[index].id);
                    break;
                }
            }
        }

        if result.len() == self.waves.len() {
            Some(result)
        } else {
            None
        }
    }
}
