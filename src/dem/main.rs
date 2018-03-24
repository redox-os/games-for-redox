#[macro_use]
extern crate extra;

use extra::io::WriteExt;

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Write};
use std::mem;

/// The number of factors.
const FACTORS: usize = 7;

/// Construct a dependency in a struct-like manner.
macro_rules! dep {
    {
        gdp: $gdp:expr,
        agb: $agb:expr,
        tax: $tax:expr,
        edu: $edu:expr,
        pov: $pov:expr,
        pop: $pop:expr,
        debt: $debt:expr,
    } => {
        [$gdp, $agb, $tax, $edu, $pov, $pop, $debt]
    };
}

/// The game.
struct Game {
    /// The factors.
    factors: [Factor; FACTORS],
    /// The aliases/identifiers of the factors.
    names: HashMap<&'static str, usize>,
}

impl Game {
    /// Create a new game state.
    fn new() -> Game {
        const GDP_ALIASES: &'static [&'static str] = &["gdp", "economy", "wealth"];
        const AGB_ALIASES: &'static [&'static str] = &["agb", "state", "budget"];
        const TAX_ALIASES: &'static [&'static str] = &["tax", "taxation"];
        const EDU_ALIASES: &'static [&'static str] = &["edu", "education", "schools"];
        const POV_ALIASES: &'static [&'static str] = &["pov", "poverty"];
        const POP_ALIASES: &'static [&'static str] = &["pop", "popularity", "popular"];
        const DEBT_ALIASES: &'static [&'static str] = &["debt", "deb", "owed"];

        let mut names = HashMap::new();
        let factors = [
            Factor {
                name: "GDP per capita",
                description: "Gross domestic product (GDP) is a monetary measure of\n\
                    the value of all final goods and services produced in a period\n\
                    (quarterly or yearly).\
                    \n\n\
                    Contributors\n\
                    -- Tax\n\
                    ++ Education\n\
                    -- Debt",
                value: 3_000,
                dependency: dep! {
                     gdp: 0.0,
                     agb: 0.0,
                     // Note that this have a negative effect due to
                     // a higher burden on the cooperations. This effect can be
                     // undone by spending them properly from the state.
                     tax: -0.2,
                     edu: 0.2,
                     pov: -0.3,
                     pop: 0.0,
                     // Too much debt lead to volatile economy.
                     debt: -0.5,
                },
                step: 0,
                prefix: "$",
                postfix: " per capita",
                alias: GDP_ALIASES,
                change: 0,
            },
            Factor {
                name: "Annual government budget",
                description: "The amount of money the state has available to spend each\n\
                    year. Bringing this in negative will indebt the state.\
                    \n\n\
                    Contributors\n\
                    ++ Tax\n\
                    ++ GDP",
                value: 0,
                dependency: dep! {
                     gdp: 0.2,
                     agb: 0.0,
                     tax: 0.7,
                     // Investment in education costs money.
                     edu: -0.6,
                     pov: 0.0,
                     pop: 0.0,
                     debt: 0.0,
                },
                step: 0,
                prefix: "$",
                postfix: " per capita",
                alias: AGB_ALIASES,
                change: 0,
            },
            Factor {
                name: "Income tax rate",
                description: "The per mille taxation rate on the citzens' incomes.\n",
                value: 0,
                dependency: dep! {
                     gdp: 0.0,
                     agb: 0.0,
                     tax: 1.0,
                     edu: 0.0,
                     pov: 0.0,
                     pop: 0.0,
                     debt: 0.0,
                },
                step: 10,
                prefix: "",
                postfix: "‰",
                alias: TAX_ALIASES,
                change: 0,
            },
            Factor {
                name: "Investment in education",
                description: "The dollars invested in education per capita on a yearly basis.",
                value: 0,
                dependency: dep! {
                     gdp: 0.0,
                     agb: 0.0,
                     tax: 0.0,
                     edu: 1.0,
                     pov: 0.0,
                     pop: 0.0,
                     debt: 0.0,
                },
                step: 40,
                prefix: "$",
                postfix: " per capita",
                alias: EDU_ALIASES,
                change: 0,
            },
            Factor {
                name: "Per mille poverty in the country",
                description: "Poverty is general scarcity, dearth, or the state of one who\n\
                    lacks a certain amount of material possessions or money. It is a multifaceted\n\
                    concept, which includes social, economic, and political elements.\
                    \n\n\
                    Contributors\n\
                    -- Education\n\
                    -- GDP\n\
                    ++ Debt\n\
                    ++ Tax",
                value: 500,
                dependency: dep! {
                     gdp: -0.1,
                     agb: 0.0,
                     tax: 0.1,
                     edu: -2.0,
                     pov: 0.0,
                     pop: 0.0,
                     debt: 0.1,
                },
                step: 0,
                prefix: "",
                postfix: "‰",
                alias: POV_ALIASES,
                change: 0,
            },
            Factor {
                name: "Popularity in the people",
                description: "The per mille number of supporters, subtracted by the per mille opposers.\
                    \n\n\
                    Contributors\n\
                    ++ GDP\n\
                    ++ Education\n\
                    ++ Budget\n\
                    -- Poverty\n\
                    -- Tax rate\n\
                    -- Debt",
                value: -400,
                dependency: dep! {
                     gdp: 0.6,
                     agb: 0.02,
                     // No one likes paying taxes.
                     tax: -0.2,
                     edu: 0.4,
                     pov: -0.6,
                     // Automatically escalate popularity.
                     pop: 0.1,
                     debt: -0.1,
                },
                step: 0,
                prefix: "",
                postfix: "‰",
                alias: POP_ALIASES,
                change: 0,
            },
            Factor {
                name: "National debt",
                description: "Percentile national debt of the GDP\
                    \n\n\
                    Contributors\n\
                    -- AGB",
                value: 80,
                dependency: dep! {
                     gdp: 0.6,
                     agb: -0.2,
                     tax: 0.0,
                     edu: 0.0,
                     pov: 0.02,
                     pop: 0.0,
                     // You cannot escape debt.
                     debt: 1.0,
                },
                step: 0,
                prefix: "",
                postfix: "%",
                alias: DEBT_ALIASES,
                change: 0,
            },
        ];

        for (n, i) in factors.iter().enumerate() {
            for &j in i.alias {
                names.insert(j, n);
            }
        }

        Game {
            factors: factors,
            names: names,
        }
    }

    /// Initialize the game and start the gameloop.
    fn init(&mut self) {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        let mut lines = BufReader::new(stdin.lock()).lines().map(|x| x.unwrap());

        stdout.writeln("<< Welcome to Dem, the commandline-based Democracy clone. 'help' for help. >>".as_bytes()).unwrap();
        loop {
            stdout.write("$ ".as_bytes()).unwrap();
            stdout.flush().unwrap();
            let i = maybe!(lines.next() => break);

            let mut args = i.split_whitespace();

            match args.next() {
                Some("inc") | Some("+") => {
                    let f = maybe!(self.get_mut(
                        maybe!(args.next() => { stdout.writeln(b"No argument given.").unwrap(); continue })
                    ) => { stdout.writeln(b"No such factor.").unwrap(); continue });

                    if f.adjustable() { f.step_up() } else {
                        stdout.writeln(b"Factor not adjustable.").unwrap();
                    }
                },
                Some("dec") | Some("-") => {
                    let f = maybe!(self.get_mut(
                        maybe!(args.next() => { stdout.writeln(b"No argument given.").unwrap(); continue })
                    ) => { stdout.writeln(b"No such factor.").unwrap(); continue });

                    if f.adjustable() { f.step_down() } else {
                        stdout.writeln(b"Factor not adjustable.").unwrap();
                    }
                },
                Some("next") | Some("n") => self.progress(),
                Some("info") | Some("i") => {
                    let factor =  maybe!(self.get(
                        maybe!(args.next() => { stdout.writeln(b"No argument given.").unwrap(); continue })
                    ) => { stdout.writeln(b"No such factor.").unwrap(); continue });

                    stdout.writeln(factor.name.as_bytes()).unwrap();
                    stdout.write(b"\nvalue: ").unwrap();
                    stdout.write(factor.prefix.as_bytes()).unwrap();
                    stdout.write(factor.value.to_string().as_bytes()).unwrap();
                    stdout.write(factor.postfix.as_bytes()).unwrap();
                    if factor.adjustable() {
                        stdout.write(b" (adjustable)").unwrap();
                    }
                    stdout.write(match factor.change {
                        0 => b" ~~~",
                        a if a < 0 => b" ---",
                        _ => b" +++",
                    }).unwrap();
                    stdout.write(b"\nalias: ").unwrap();

                    stdout.write(factor.alias[0].as_bytes()).unwrap();
                    for alias in &factor.alias[1..] {
                        stdout.write(b", ").unwrap();
                        stdout.write(alias.as_bytes()).unwrap();
                    }

                    stdout.write(b"\n\n").unwrap();
                    stdout.writeln(factor.description.as_bytes()).unwrap();
                },
                Some("ls") | Some("l") => {
                    stdout.writeln(b"\
                        gdp  - GDP per capita\n\
                        agb  - State budget\n\
                        tax  - Income tax\n\
                        edu  - Education\n\
                        pov  - Poverty\n\
                        pop  - Popularity\n\
                        debt - National debt").unwrap();
                },
                Some("exit") => break,
                Some("help") | Some("h") => {
                    stdout.writeln(b"\
                        ls   - List factors\n\
                        inc  - Increment factor\n\
                        dec  - Decrement factor\n\
                        info - Get info about a given factor\n\
                        next - Go to next round\n
                        exit - Exit the program").unwrap();
                },
                _ => {
                    stdout.writeln(b"No such command.").unwrap();
                },
            }
        }
    }

    /// Look up a factor through one of its aliases.
    fn get(&mut self, s: &str) -> Option<&Factor> {
        self.factors.get(*maybe!(self.names.get(s)))
    }

    /// Look up a factor through one of its aliases.
    ///
    /// Returns a mutable borrow.
    fn get_mut(&mut self, s: &str) -> Option<&mut Factor> {
        let id = self.names.get(s);
        self.factors.get_mut(*maybe!(id))
    }

    /// Get an array of the current values of the factors.
    fn values(&self) -> [i64; FACTORS] {
        let mut values: [i64; FACTORS] = unsafe { mem::uninitialized() };
        for (n, i) in self.factors.iter().enumerate() {
            values[n] = i.value;
        }
        values
    }

    /// Progress, e.g. update all the values.
    ///
    /// This will enter the next round.
    fn progress(&mut self) {
        let val = self.values();

        for (n, i) in self.factors.iter_mut().enumerate() {
            i.value = i.dependency.iter().enumerate().map(|(m, j)| val[m] as f32 * j).sum::<f32>() as i64;
            i.change = i.value - val[n];
        }
    }
}

/// An "factor", i.e. a value or property which can or cannot be adjusted, and might be dependent
/// on other factors.
#[derive(Copy, Clone)]
struct Factor {
    /// The factor's name.
    name: &'static str,
    /// The description of this factor.
    description: &'static str,
    /// The value of this factor.
    value: i64,
    /// The coefficients of the dependency.
    ///
    /// The value of this factor is calculated as a linear combination of all the factors (that is,
    /// the factors new value is defined by multiplying the vector with the dependency matrix).
    /// These values are the coefficients, which will be multiplied with the respective factors,
    /// adding up to the new value.
    dependency: [f32; FACTORS],
    /// The "step" of the value, that is how much the value will be incremented/decremented at a
    /// time.
    step: i64,
    /// The prefix to the value.
    ///
    /// This will be put before the value is printed.
    prefix: &'static str,
    /// The postfix to the value.
    ///
    /// This will be put after the value is printed.
    postfix: &'static str,
    /// The aliases/identifiers of this value.
    alias: &'static [&'static str],
    /// The change since last time.
    change: i64,
}

impl Factor {
    /// Add the step to the value.
    fn step_up(&mut self) {
        self.value += self.step;
    }

    /// Subtract the step from the value.
    fn step_down(&mut self) {
        self.value -= self.step;
    }

    /// Is this factor adjustable?
    fn adjustable(&self) -> bool {
        self.step != 0
    }
}

fn main() {
    let mut game = Game::new();
    game.init();
}
