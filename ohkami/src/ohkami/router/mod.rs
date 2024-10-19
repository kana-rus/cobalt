#![allow(non_snake_case)]

mod trie;
pub(crate) use trie::TrieRouter;

mod radix;
pub(crate) use radix::RadixRouter;

use std::{collections::VecDeque, iter::Peekable, str::Chars, mem::MaybeUninit};


#[derive(Clone, Debug)]
pub struct RouteSections {
    litreral: &'static str,
    sections: VecDeque<RouteSection>,
}
const _: () = {
    impl RouteSections {
        pub(crate) fn from_literal(route: &'static str) -> Self {
            if route.is_empty() {panic!("Found an empty route: `{route}`")}
            if !route.starts_with('/') {panic!("Routes must start with '/': `{route}`")}

            if route == "/" {
                return Self {
                    litreral: route,
                    sections: VecDeque::new()
                }
            }

            let mut sections = VecDeque::new();
            for section in {let mut s = route.split('/'); s.next(); s} {
                let section = match RouteSection::new(section.as_bytes()) {
                    Err(e) => panic!("{e}: `{route}`"),
                    Ok(rs) => rs,
                };
                sections.push_back(section)
            }

            Self { litreral:route, sections }
        }

        pub(crate)  fn literal(&self) -> &'static str {
            self.litreral
        }
    }

    impl IntoIterator for RouteSections {
        type Item = <VecDeque<RouteSection> as IntoIterator>::Item;
        type IntoIter = <VecDeque<RouteSection> as IntoIterator>::IntoIter;
        fn into_iter(self) -> Self::IntoIter {self.sections.into_iter()}
    }
};

#[derive(Clone)]
pub enum RouteSection {
    Static(&'static [u8]),
    Param,
}
const _: () = {
    impl RouteSection {
        pub(crate) fn new(section_bytes: &'static [u8]) -> Result<Self, String> {
            let mut section_chars = std::str::from_utf8(section_bytes).unwrap().chars().peekable();

            fn validate_section_name(mut name: Peekable<Chars>) -> Result<(), String> {
                let is_invalid_head_or_tail_char = |c: char| !/* NOT */ matches!(c,
                    '0'..='9' | 'a'..='z' | 'A'..='Z'
                );

                let is_invalid_char = |c: char| !/* NOT */ matches!(c,
                    '.' | '-' | '_' | '0'..='9' | 'a'..='z' | 'A'..='Z'
                );

                let Some(head) = name.next() else {return Err(format!("Found an empty section name"))};
                if is_invalid_head_or_tail_char(head) {
                    return Err(format!("Path section can't start with '{head}'"))
                }

                let Some(tail) = name.next_back() else {return Ok(())};
                if is_invalid_head_or_tail_char(tail) {
                    return Err(format!("Path section can't end with '{tail}'"))
                }

                for c in name {
                    if is_invalid_char(c) {
                        return Err(format!("Path section can't contain '{c}'"))
                    }
                }

                Ok(())
            }

            match section_chars.peek() {
                None => Err(format!("Found an empty route section_chars")),
                Some(':') => {
                    let _/* colon */ = section_chars.next();
                    let _/* validation */ = validate_section_name(section_chars)?;
                    Ok(Self::Param)
                },
                _ => {
                    let _/* validation */ = validate_section_name(section_chars)?;
                    Ok(Self::Static(section_bytes))
                }
            }
        }
    }

    impl std::fmt::Debug for RouteSection {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Param         => f.write_str(":Param"),
                Self::Static(bytes) => f.write_str(std::str::from_utf8(bytes).unwrap()),
            }
        }
    }
};


pub(crate) struct PathParams<'req> {
    list: [MaybeUninit<&'req [u8]>; PARAMS_LIMIT],
    next: u8,
}

const PARAMS_LIMIT: usize = 2;

const _: () = {
    impl<'req> PathParams<'req> {
        pub(crate) const fn new() -> Self {
            Self {
                list: [const {MaybeUninit::uninit()}; PARAMS_LIMIT],
                next: 0
            }
        }

        pub(crate) fn push(&mut self, param: &'req [u8]) {
            let next = self.next as usize;
            if next == PARAMS_LIMIT {
                #[cfg(debug_assertions)] {
                    crate::warning!("throwing away a path param `{}`: Ohkami doesn't handle more than two path params currently", param.escape_ascii())
                }
                return
            }
            // SAFETY: next < PARAMS_LIMIT = self.list.len()
            unsafe {self.list.get_unchecked_mut(next)}.write(param);
            self.next += 1;
        }

        // SAFETY: `self` must have called `push` at least once
        pub(crate) unsafe fn assume_init_one(self) -> &'req [u8] {
            self.list.get_unchecked(0).assume_init_ref()
        }

        // SAFETY: `self` must have called `push` twice
        pub(crate) unsafe fn assume_init_two(self) -> (&'req [u8], &'req [u8]) {
            (
                self.list.get_unchecked(0).assume_init_ref(),
                self.list.get_unchecked(1).assume_init_ref()
            )
        }
    }
};
