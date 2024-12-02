//! Most parts are based on [chrono](https://github.com/chronotope/chrono); MIT.


/// Current datetime by **IMF-fixdate** format like `Sun, 06 Nov 1994 08:49:37 GMT`, used in `Date` header.
/// 
/// (reference：[https://datatracker.ietf.org/doc/html/rfc9110#name-date-time-formats](https://datatracker.ietf.org/doc/html/rfc9110#name-date-time-formats))
#[inline(always)] pub fn imf_fixdate(unix_timestamp: u64) -> String {
    // SAFETY: into_imf_fixdate() always generates valid UTF-8 bytes
    unsafe {String::from_utf8_unchecked(Vec::from(
        UTCDateTime::from_unix_timestamp(unix_timestamp).into_imf_fixdate()
    ))}
}

pub const IMF_FIXDATE_LEN: usize = str::len("Sun, 06 Nov 1994 08:49:37 GMT");
        
/// date time on UTC *to the second*
pub struct UTCDateTime {
    date: Date,
    time: Time,
}
impl UTCDateTime {
    #[inline]
    pub fn from_unix_timestamp(unix_timestamp: u64) -> Self {
        let secs = unix_timestamp as i64;

        let days = secs.div_euclid(86_400);
        let secs = secs.rem_euclid(86_400);

        let date = Date::from_days(days as i32 + 719_163);
        let time = Time::from_seconds(secs as u32);

        Self { date, time }
    }

    pub fn into_imf_fixdate(self) -> [u8; IMF_FIXDATE_LEN] {
        const SHORT_WEEKDAYS: [&[u8; 3]; 7 ] = [b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"];
        const SHORT_MONTHS:   [&[u8; 3]; 12] = [b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec"];

        let mut buf = [std::mem::MaybeUninit::<u8>::uninit(); IMF_FIXDATE_LEN];
        let mut i = 0;

        macro_rules! fill {
            ($bytes:expr) => {
                for &b in $bytes {
                    fill!(@b)
                }
            };
            (@100> $byte:expr) => {{
                #[cfg(debug_assertions)] {
                    assert!($byte < 100)
                }
                fill!(@b'0' + $byte/10);
                fill!(@b'0' + $byte%10);
            }};
            (@$byte:expr) => {{
                #[cfg(debug_assertions)] {
                    assert!(i < buf.len())
                }
                unsafe {buf.get_unchecked_mut(i)}.write($byte);
                i += 1;
            }};
        }

        let UTCDateTime { date, time } = self;
        let day  = date.day() as u8;
        let year = date.year();
        let (hour, min, sec) = time.hms();

        fill!(*unsafe {SHORT_WEEKDAYS.get_unchecked(date.weekday().num_days_from_sunday() as usize)});
        fill!(b", ");
        if day < 10 {
            fill!(@b'0'); fill!(@b'0' + day)
        } else {
            fill!(@100> day)
        }
        fill!(@b' ');
        fill!(*unsafe {SHORT_MONTHS.get_unchecked(date.month_index() as usize)});
        fill!(@b' ');
        fill!(@100> (year/100) as u8);
        fill!(@100> (year%100) as u8);
        fill!(@b' ');
        fill!(@100> hour as u8);
        fill!(@b':');
        fill!(@100> min as u8);
        fill!(@b':');
        fill!(@100> sec as u8);
        fill!(b" GMT");

        #[cfg(debug_assertions)] {
            assert_eq!(i, buf.len())
        }

        unsafe {
            // SAFETY:
            // * The caller guarantees that all elements of the array are initialized
            // * `MaybeUninit<T>` and T are guaranteed to have the same layout
            // * `MaybeUninit` does not drop, so there are no double-frees
            // And thus the conversion is safe
            std::mem::transmute::<_, [u8; IMF_FIXDATE_LEN]>(buf)
        }
    }
}

/// (year << 13) | of
#[derive(Debug, PartialEq)]
struct Date(i32);
impl Date {
    fn from_days(days: i32) -> Self {
        const YEAR_DELTAS: &[u8; 401] = &[
            0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8,
            8, 9, 9, 9, 9, 10, 10, 10, 10, 11, 11, 11, 11, 12, 12, 12, 12, 13, 13, 13, 13, 14, 14, 14, 14,
            15, 15, 15, 15, 16, 16, 16, 16, 17, 17, 17, 17, 18, 18, 18, 18, 19, 19, 19, 19, 20, 20, 20, 20,
            21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23, 24, 24, 24, 24, 25, 25, 25, // 100
            25, 25, 25, 25, 25, 26, 26, 26, 26, 27, 27, 27, 27, 28, 28, 28, 28, 29, 29, 29, 29, 30, 30, 30,
            30, 31, 31, 31, 31, 32, 32, 32, 32, 33, 33, 33, 33, 34, 34, 34, 34, 35, 35, 35, 35, 36, 36, 36,
            36, 37, 37, 37, 37, 38, 38, 38, 38, 39, 39, 39, 39, 40, 40, 40, 40, 41, 41, 41, 41, 42, 42, 42,
            42, 43, 43, 43, 43, 44, 44, 44, 44, 45, 45, 45, 45, 46, 46, 46, 46, 47, 47, 47, 47, 48, 48, 48,
            48, 49, 49, 49, // 200
            49, 49, 49, 49, 49, 50, 50, 50, 50, 51, 51, 51, 51, 52, 52, 52, 52, 53, 53, 53, 53, 54, 54, 54,
            54, 55, 55, 55, 55, 56, 56, 56, 56, 57, 57, 57, 57, 58, 58, 58, 58, 59, 59, 59, 59, 60, 60, 60,
            60, 61, 61, 61, 61, 62, 62, 62, 62, 63, 63, 63, 63, 64, 64, 64, 64, 65, 65, 65, 65, 66, 66, 66,
            66, 67, 67, 67, 67, 68, 68, 68, 68, 69, 69, 69, 69, 70, 70, 70, 70, 71, 71, 71, 71, 72, 72, 72,
            72, 73, 73, 73, // 300
            73, 73, 73, 73, 73, 74, 74, 74, 74, 75, 75, 75, 75, 76, 76, 76, 76, 77, 77, 77, 77, 78, 78, 78,
            78, 79, 79, 79, 79, 80, 80, 80, 80, 81, 81, 81, 81, 82, 82, 82, 82, 83, 83, 83, 83, 84, 84, 84,
            84, 85, 85, 85, 85, 86, 86, 86, 86, 87, 87, 87, 87, 88, 88, 88, 88, 89, 89, 89, 89, 90, 90, 90,
            90, 91, 91, 91, 91, 92, 92, 92, 92, 93, 93, 93, 93, 94, 94, 94, 94, 95, 95, 95, 95, 96, 96, 96,
            96, 97, 97, 97, 97, // 400+1
        ];

        let days = days + 365;
        let year_div_400 = days.div_euclid(146_097);
        let cycle = days.rem_euclid(146_097) as u32;

        let mut year_mod_400 = cycle / 365;
        let mut ordinal = cycle % 365 + 1;
        let delta = unsafe {*YEAR_DELTAS.get_unchecked(year_mod_400 as usize)} as u32;
        if ordinal <= delta {
            year_mod_400 -= 1;
            ordinal += 365 - unsafe {*YEAR_DELTAS.get_unchecked(year_mod_400 as usize)} as u32;
        } else {
            ordinal -= delta;
        }

        let flags = YearFlag::from_year(year_mod_400 as i32);
        Self::from_ordinal_and_flags(year_div_400 * 400 + year_mod_400 as i32, ordinal, flags)
    }
    #[inline(always)] fn from_ordinal_and_flags(
        year:    i32,
        ordinal: u32,
        flag:    YearFlag,
    ) -> Date {
        #[cfg(debug_assertions)] assert!({
            const MAX_YEAR: i32 = i32::MAX >> 13;
            const MIN_YEAR: i32 = i32::MIN >> 13;

            year >= MIN_YEAR &&
            year <= MAX_YEAR &&
            YearFlag::from_year(year).0 == flag.0
        });

        let of = Of::new(ordinal, flag);
        Self((year << 13) | (of.0 as i32))
    }

    #[inline(always)] const fn year(&self) -> i32 {
        self.0 >> 13
    }
    #[inline(always)] fn month_index(&self) -> u32 {
        self.of().to_mdf().month() - 1
    }
    #[inline(always)] fn day(&self) -> u32 {
        self.of().to_mdf().day()
    }
    #[inline(always)] const fn weekday(&self) -> Weekday {
        self.of().weekday()
    }
    #[inline(always)] const fn of(&self) -> Of {
        Of::from_date(self.0)
    }
}

#[derive(Debug, PartialEq)]
struct Time {
    secs: u32,
}
impl Time {
    #[inline(always)] const fn from_seconds(secs: u32) -> Self {
        #[cfg(debug_assertions)] assert! {
            secs < 86_400
        }

        Self { secs }
    }
    #[inline] const fn hms(&self) -> (u32, u32, u32) {
        let sec = self.secs % 60;
        let mins = self.secs / 60;
        let min = mins % 60;
        let hour = mins / 60;
        (hour, min, sec)
    }
}

#[derive(Clone, Copy)]
struct YearFlag(u8);
impl YearFlag {
    #[inline(always)] fn from_year(year: i32) -> Self {
        const A:  YearFlag = YearFlag(0o15);
        const AG: YearFlag = YearFlag(0o05);
        const B:  YearFlag = YearFlag(0o14);
        const BA: YearFlag = YearFlag(0o04);
        const C:  YearFlag = YearFlag(0o13);
        const CB: YearFlag = YearFlag(0o03);
        const D:  YearFlag = YearFlag(0o12);
        const DC: YearFlag = YearFlag(0o02);
        const E:  YearFlag = YearFlag(0o11);
        const ED: YearFlag = YearFlag(0o01);
        const F:  YearFlag = YearFlag(0o17);
        const FE: YearFlag = YearFlag(0o07);
        const G:  YearFlag = YearFlag(0o16);
        const GF: YearFlag = YearFlag(0o06);

        const YEAR_TO_FLAG: &[YearFlag; 400] = &[
            BA, G, F, E, DC, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA,
            G, F, E, DC, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G,
            F, E, DC, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F,
            E, DC, B, A, G, FE, D, C, B, AG, F, E, D, // 100
            C, B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC,
            B, A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B,
            A, G, FE, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A,
            G, FE, D, C, B, AG, F, E, D, CB, A, G, F, // 200
            E, D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE,
            D, C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D,
            C, B, AG, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C,
            B, AG, F, E, D, CB, A, G, F, ED, C, B, A, // 300
            G, F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C, B, AG,
            F, E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C, B, AG, F,
            E, D, CB, A, G, F, ED, C, B, A, GF, E, D, C, BA, G, F, E, DC, B, A, G, FE, D, C, B, AG, F, E,
            D, CB, A, G, F, ED, C, B, A, GF, E, D, C, // 400
        ];

        unsafe {*YEAR_TO_FLAG.get_unchecked(year.rem_euclid(400) as usize)}
    }
}

#[derive(Clone, Copy)]
struct Of(u32);
impl Of {
    #[inline] const fn new(ordinal: u32, YearFlag(flag): YearFlag) -> Self {
        let of = Self((ordinal << 4) | flag as u32);
        #[cfg(debug_assertions)] assert!({
            const MIN_OL: u32 = 1 << 1;
            const MAX_OL: u32 = 366 << 1; // `(366 << 1) | 1` would be day 366 in a non-leap year

            let ol = of.0 >> 3;
            MIN_OL <= ol && ol <= MAX_OL
        });
        of
    }
    #[inline] const fn from_date(date: i32) -> Self {
        Self((date & 0b1_1111_1111_1111) as u32)
    }
    #[inline] const fn weekday(&self) -> Weekday {
        let Of(of) = *self;
        Weekday::from_u32_mod7((of >> 4) + (of & 0b111))
    }
    #[inline] fn to_mdf(&self) -> Mdf {
        Mdf::from_of(*self)
    }
}

struct Mdf(u32);
impl Mdf {
    #[inline] const fn month(&self) -> u32 {
        let Mdf(mdf) = *self;
        mdf >> 9
    }
    #[inline] const fn day(&self) -> u32 {
        let Mdf(mdf) = *self;
        (mdf >> 4) & 0b1_1111
    }

    #[inline] fn from_of(Of(of): Of) -> Mdf {
        const MAX_OL: u32 = 366 << 1; // `(366 << 1) | 1` would be day 366 in a non-leap year
        const OL_TO_MDL: &[u8; MAX_OL as usize + 1] = &[
            0, 0, // 0
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
            64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, // 1
            66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
            66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66,
            66, 66, 66, 66, 66, 66, 66, 66, 66, // 2
            74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72,
            74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72,
            74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, 74, 72, // 3
            76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74,
            76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74,
            76, 74, 76, 74, 76, 74, 76, 74, 76, 74, 76, 74, // 4
            80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78,
            80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78,
            80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, 80, 78, // 5
            82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80,
            82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80,
            82, 80, 82, 80, 82, 80, 82, 80, 82, 80, 82, 80, // 6
            86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84,
            86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84,
            86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, 86, 84, // 7
            88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86,
            88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86,
            88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, 88, 86, // 8
            90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88,
            90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88,
            90, 88, 90, 88, 90, 88, 90, 88, 90, 88, 90, 88, // 9
            94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92,
            94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92,
            94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, 94, 92, // 10
            96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94,
            96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94,
            96, 94, 96, 94, 96, 94, 96, 94, 96, 94, 96, 94, // 11
            100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100,
            98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98,
            100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100, 98, 100,
            98, // 12
        ];

        let ol = of >> 3;
        if ol <= MAX_OL {
            // Array is indexed from `[1..=MAX_OL]`, with a `0` index having a meaningless value.
            Mdf(of + ((unsafe {*OL_TO_MDL.get_unchecked(ol as usize)} as u32) << 3))
        } else {
            // Panicking here would be reasonable, but we are just going on with a safe value.
            Mdf(0)
        }
    }
}

#[derive(Clone, Copy)]
enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}
impl Weekday {
    #[inline] const fn from_u32_mod7(n: u32) -> Self {
        match n % 7 {
            0 => Self::Mon,
            1 => Self::Tue,
            2 => Self::Wed,
            3 => Self::Thu,
            4 => Self::Fri,
            5 => Self::Sat,
            _ => Self::Sun,
        }
    }

    #[inline] const fn num_days_from_sunday(&self) -> u32 {
        (*self as u32 + 7 - Self::Sun as u32) % 7
    }
}


#[cfg(test)] mod test {
    #[test] fn test_now() {
        fn correct_now() -> String {
            let mut output_bytes = std::process::Command::new("/usr/bin/date")
                .env("LANG", "en_US")
                .arg("+'%a, %d %b %Y %H:%M:%S GMT'")
                .arg("-u")
                .output().unwrap()
                .stdout;

            // `output_bytes` is like
            // 
            // ```escape_ascii:
            // 'Sat, 30 Dec 2023 19:05:26 GMT'\n
            // ```
            output_bytes.rotate_left(1);
            output_bytes.truncate(output_bytes.len() - 3);
            // Here
            // 
            // ```escape_ascii:
            // Sat, 30 Dec 2023 19:05:26 GMT
            // ```

            String::from_utf8(output_bytes).unwrap()
        }

        use std::time::{SystemTime, UNIX_EPOCH};
        let system_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before Unix epoch");
        let (expected, n) = (correct_now(), super::imf_fixdate(system_now.as_secs()));
        assert_eq!(expected, n);
    }
}
