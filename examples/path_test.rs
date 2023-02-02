fn main() {
    let path = normalize("/greet/**/hello-{name}/{text}*");
    let out = tokenize(path);
    println!("{:?}", out);

    let _match = match_path(&out, "/greet/hi/hello-world/this-is-a-test");
    println!("{:?}", _match);
}

fn tokenize(str: &str) -> Vec<Tokens> {
    let chars = str.chars().collect::<Vec<_>>();
    let mut out = Vec::new();
    let mut working = String::new();

    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '{' => {
                if !working.is_empty() {
                    out.push(Tokens::Normal(working));
                    working = String::new();
                }
            }
            '}' => {
                out.push(Tokens::Param(working));
                working = String::new();
            }
            '*' if i + 1 < chars.len() && chars[i + 1] == '*' => {
                if !working.is_empty() {
                    out.push(Tokens::Normal(working));
                    working = String::new();
                }
                out.push(Tokens::AnyAfter);
                i += 1;
            }
            '*' => {
                if !working.is_empty() {
                    out.push(Tokens::Normal(working));
                    working = String::new();
                }
                out.push(Tokens::Any);
            }
            i => working.push(i),
        }
        i += 1;
    }

    if !working.is_empty() {
        out.push(Tokens::Normal(working));
    }

    out
}

fn match_path(path: &[Tokens], inp: &str) -> Option<Vec<(String, String)>> {
    let mut out = Vec::new();
    let inp = normalize(inp);
    let mut inp_index = 0;

    for i in path {
        println!("{:?} {}", i, &inp[inp_index..]);
        match i {
            Tokens::Normal(x) => {
                if inp[inp_index..].starts_with(x) {
                    inp_index += x.len();
                    continue;
                }
                return None;
            }
            Tokens::Param(x) => {
                let end = inp[inp_index..].find('/').unwrap_or(inp.len() - inp_index);
                if end == 0 {
                    return None;
                }
                out.push((x.to_owned(), inp[inp_index..inp_index + end].to_owned()));
                inp_index += end;
            }
            Tokens::AnyAfter => {
                loop {
                    let end = inp[inp_index..].find('/').unwrap_or(inp.len() - inp_index);
                    if end == 0 {
                        return None;
                    }
                    inp_index += end;
                    if inp_index == inp.len() {
                        return Some(out);
                    }
                }
            },
            Tokens::Any => {}
        }
    }

    Some(out)
}

#[derive(Debug)]
enum Tokens {
    Normal(String),
    Param(String),
    AnyAfter,
    Any,
}

fn normalize(inp: &str) -> &str {
    inp.trim_start_matches('/').trim_end_matches('/')
}
