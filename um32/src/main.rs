use std::{
    collections::{BTreeSet, HashMap},
    fs,
    io::{self, Read, Write},
};

fn main() {
    let mut st = State::new(
        &fs::read(
            fs::canonicalize(std::env::args().nth(1).unwrap_or_else(|| help()))
                .expect("could not canonicalize argument"),
        )
        .expect("could not read file"),
        match std::env::args().nth(2) {
            Some(x) => Some(fs::File::create(x).expect("cannot create file")),
            None => None,
        },
    );
    while st.advance() {}
}

fn help() -> ! {
    println!(
        "usage: {} <filename> [output-file]",
        std::env::current_exe()
            .unwrap_or_else(|_| "um32".into())
            .display()
    );
    std::process::exit(-1);
}

fn read(x: &[u8]) -> Vec<u32> {
    x.chunks(4)
        .map(|i| u32::from_be_bytes(i.try_into().unwrap()))
        .collect()
}

struct State {
    mem: HashMap<u32, Vec<u32>>,
    free: BTreeSet<u32>,
    reg: [u32; 8],
    ip: usize,
    least: u32,
    inp: io::Bytes<io::Stdin>,
    out: Option<fs::File>,
}

impl State {
    fn new(b: &[u8], out: Option<fs::File>) -> State {
        State {
            mem: HashMap::from([(0, read(b))]),
            free: BTreeSet::new(),
            reg: [0; 8],
            ip: 0,
            least: 1,
            out,
            inp: io::stdin().bytes(),
        }
    }

    fn advance(&mut self) -> bool {
        let instr = self.mem[&0][self.ip];
        self.ip += 1;
        let op = instr >> 28;
        if op > 13 || op == 7 {
            return false;
        }
        let c = self.reg[(instr & 7) as usize];
        let b = self.reg[((instr >> 3) & 7) as usize];
        let a = &mut self.reg[((instr >> 6) & 7) as usize];
        match op {
            0 => {
                if c > 0 {
                    *a = b;
                }
            }
            1 => *a = self.mem[&b][c as usize],
            2 => self.mem.get_mut(&a).unwrap()[b as usize] = c,
            3 => *a = b.wrapping_add(c),
            4 => *a = b.wrapping_mul(c),
            5 => *a = b / c,
            6 => *a = !(b & c),

            _ => {}
        }
        if op < 7 {
            return true;
        }
        let c = (instr & 7) as usize;
        let b = ((instr >> 3) & 7) as usize;
        match op {
            8 => self.alloc(b, c),
            9 => self.free(self.reg[c]),
            10 => {
                let byte: u8 = self.reg[c].try_into().unwrap();
                if let Some(ref mut stdout) = self.out {
                    stdout.write_all(&[byte]).unwrap();
                    stdout.flush().unwrap();
                } else {
                    std::io::stdout().write_all(&[byte]).unwrap();
                }
            }
            11 => {
                if self.out.is_none() {
                    std::io::stdout().flush().unwrap();
                }
                self.reg[c] = match self.inp.next() {
                    Some(Ok(x)) => x.into(),
                    _ => u32::MAX,
                }
            }
            12 => {
                if self.reg[b] > 0 {
                    self.mem.insert(0, self.mem[&self.reg[b]].clone());
                }
                self.ip = self.reg[c] as _;
            }
            13 => {
                let a = ((instr >> 25) & 7) as usize;
                let x = instr & (1 << 25) - 1;
                self.reg[a] = x;
            }

            _ => unreachable!("{}", op),
        }
        true
    }

    fn alloc(&mut self, b: usize, c: usize) {
        self.mem.insert(self.least, vec![0; self.reg[c] as _]);
        self.reg[b] = self.least;
        if let Some(x) = self.free.pop_first() {
            self.least = x;
        } else {
            self.least += 1;
            while self.mem.contains_key(&self.least) {
                self.least += 1;
            }
        }
    }

    fn free(&mut self, a: u32) {
        self.mem.remove(&a).unwrap();
        if a < self.least {
            self.free.insert(self.least);
            self.least = a;
        } else {
            self.free.insert(a);
        }
    }
}
/*
struct BTree<T: Ord> {
    node: Option<T>,
    ch: Vec<BTree<T>>,
}

impl<T: Ord> BTree<T> {
    fn pop(&mut self) -> Option<T> {
        if let Some(p) = self.ch.pop() {
            let mut ch = p.ch;
            ch.reverse();
            self.ch.append(&mut ch);
            Some(p.node.unwrap())
        } else {
            self.node.take()
        }
    }
}*/
