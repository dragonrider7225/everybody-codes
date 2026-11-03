use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead, BufReader},
    str::FromStr,
};

struct Recipe {
    product: String,
    ingredients: Vec<String>,
}

impl FromStr for Recipe {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((product, ingredients)) = s.split_once(':') else {
            return Err(format!("No colon in {s:?}"));
        };
        let product = product.to_string();
        let ingredients = ingredients.split(',').map(|s| s.to_string()).collect();
        Ok(Self {
            product,
            ingredients,
        })
    }
}

/// Finds all strings `s` that are produced by any positive number of recursive calls to `products`
/// starting at `ingredient` other than `ingredient` itself.
fn find_all_products<'ingredient, 'products, I>(
    ingredient: &'ingredient str,
    mut products: impl FnMut(&'products str) -> I,
) -> impl Iterator<Item = &'products str>
where
    'ingredient: 'products,
    I: Iterator<Item = &'products str>,
{
    let mut found_products = HashSet::new();
    let mut to_evaluate = HashSet::<_>::from_iter([ingredient]);
    let mut scratch = HashSet::new();
    while !to_evaluate.is_empty() {
        found_products.extend(to_evaluate.iter().copied());
        std::mem::swap(&mut to_evaluate, &mut scratch);
        to_evaluate.extend(
            scratch
                .drain()
                .flat_map(&mut products)
                .filter(|product| !found_products.contains(product)),
        );
    }
    found_products.remove(ingredient);
    found_products.into_iter()
}

fn part1(input: &mut dyn BufRead) -> io::Result<usize> {
    let ingredients = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                let recipe = line
                    .parse::<Recipe>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Ok((recipe.product, recipe.ingredients))
            })
        })
        .collect::<io::Result<HashMap<String, Vec<String>>>>()?;
    /*
    {
        use std::{fs::OpenOptions, io::Write};

        let mut ingredients_map_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("lotw.6.1.gv")?;
        writeln!(ingredients_map_file, "strict digraph ingredients {{")?;
        for (ingredient, parents) in &ingredients {
            write!(ingredients_map_file, "    {ingredient:?} -> {{ ")?;
            for parent in parents {
                write!(ingredients_map_file, "{parent:?}; ")?;
            }
            writeln!(ingredients_map_file, "}};")?;
        }
        writeln!(ingredients_map_file, "}}")?;
    }
    */
    let mut found_ingredients = HashSet::new();
    let mut to_decompose = HashSet::<_>::from_iter(["NULL"]);
    while !to_decompose.is_empty() {
        #[cfg(debug_assertions)]
        {
            eprintln!("Decomposing {} ingredients", to_decompose.len());
            let s = format!("{to_decompose:?}");
            for line in s.lines() {
                eprintln!("    {line}");
            }
        }
        found_ingredients.extend(to_decompose.iter().copied());
        std::mem::take(&mut to_decompose)
            .into_iter()
            .for_each(|ingredient| match ingredients.get(ingredient) {
                None => {
                    #[cfg(debug_assertions)]
                    eprintln!("{ingredient:?} has no ingredients");
                }
                Some(parents) => {
                    #[cfg(debug_assertions)]
                    eprintln!("The ingredients of {ingredient:?} are {parents:?}");
                    to_decompose.extend(
                        parents
                            .iter()
                            .map(|s| &**s)
                            .filter(|parent| !found_ingredients.contains(parent)),
                    );
                }
            });
    }
    // `NULL` is never counted as an ingredient in `NULL`, despite cycles not being forbidden.
    Ok(found_ingredients.len() - 1)
}

fn part2(input: &mut dyn BufRead) -> io::Result<usize> {
    let products = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Recipe>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
        })
        .try_fold(HashMap::new(), |mut acc, recipe| -> io::Result<_> {
            let recipe = recipe?;
            let acc_ref = &mut acc;
            recipe.ingredients.into_iter().for_each(move |ingredient| {
                acc_ref
                    .entry(ingredient)
                    .or_insert(vec![])
                    .push(recipe.product.clone());
            });
            Ok(acc)
        })?;
    Ok(find_all_products("NULL", |ingredient| {
        products.get(ingredient).into_iter().flatten().map(|s| &**s)
    })
    .count())
}

fn part3(input: &mut dyn BufRead) -> io::Result<usize> {
    let products = input
        .lines()
        .map(|line| {
            line.and_then(|line| {
                line.parse::<Recipe>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            })
        })
        .try_fold(HashMap::new(), |mut acc, recipe| -> io::Result<_> {
            let recipe = recipe?;
            let acc_ref = &mut acc;
            recipe.ingredients.into_iter().for_each(move |ingredient| {
                acc_ref
                    .entry(ingredient)
                    .or_insert(vec![])
                    .push(recipe.product.clone());
            });
            Ok(acc)
        })?;
    let mut products = |ingredient| products.get(ingredient).into_iter().flatten().map(|s| &**s);
    let null1_products = find_all_products("NULLv1", &mut products).collect::<HashSet<_>>();
    let null2_products = find_all_products("NULLv2", &mut products).collect::<HashSet<_>>();
    let null3_products = find_all_products("NULLv3", &mut products).collect::<HashSet<_>>();
    Ok(null1_products
        .intersection(&null2_products)
        .chain(null1_products.intersection(&null3_products))
        .chain(null2_products.intersection(&null3_products))
        .collect::<HashSet<_>>()
        .len())
}

pub(super) fn run() -> io::Result<()> {
    {
        println!("Lord of the wings Quest 6 Part 1");
        println!(
            "{}",
            part1(&mut BufReader::new(File::open(
                "lord-of-the-wings_06-1.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 6 Part 2");
        println!(
            "{}",
            part2(&mut BufReader::new(File::open(
                "lord-of-the-wings_06-2.txt"
            )?))?
        );
    }
    {
        println!("Lord of the wings Quest 6 Part 3");
        println!(
            "{}",
            part3(&mut BufReader::new(File::open(
                "lord-of-the-wings_06-3.txt"
            )?))?
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_part1() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "BRDS:FLWR,SLTT,YSTS\n",
            "FLWR:SLTT,YSTS\n",
            "SLTT:YSTS,MLKK,BTTR\n",
            "NULL:BRDS,FLWR,SLTT\n",
            "YSTS:BRDS,FLWR\n",
            "MLKK:SPCS,NULL\n",
            "HNYN:BRDS,FLWR,SLTT,YSTS\n",
        );
        let expected = 7;
        let actual = part1(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part2() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "BRDS:FLWR,SLTT,YSTS\n",
            "FLWR:SLTT,YSTS\n",
            "SLTT:YSTS,MLKK,BTTR\n",
            "NULL:BRDS,FLWR,SLTT\n",
            "YSTS:BRDS,FLWR\n",
            "MLKK:SPCS,NULL\n",
            "HNYN:BRDS,FLWR,SLTT,YSTS\n",
        );
        let expected = 6;
        let actual = part2(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_part3() -> io::Result<()> {
        const TEST_DATA: &str = concat!(
            "MEAL:BRDS,CAKE,DOUG\n",
            "BRDS:DOUG,FLWR\n",
            "CAKE:DOUG,FLWR,NULLv2,NULLv3\n",
            "DOUG:FLWR,YSTS,SLTT,NULLv1\n",
            "FLWR:BRDS,DOUG\n",
            "YSTS:SPCS\n",
            "SOUP:BRDS,CAKE,DOUG,FLWR\n",
        );
        let expected = 3;
        let actual = part3(&mut Cursor::new(TEST_DATA))?;
        assert_eq!(expected, actual);
        Ok(())
    }
}
