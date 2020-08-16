use crate::prelude::*;

macro_rules! pixel_action { 
    (
        $id:expr; $output_info:ident $(& $idents:ident)* > $o:ident
        $($precoli:ident = $precols:expr),*; $($postcol:block,)? 
        $(|$frame:ident| [$r:ident $g:ident $b:ident])+ $($px:ident)? $action:block
        $(=> $end:expr)?
    ) => {{
        #[allow(unused_mut)]
        let mut $o = vec![$id];
        for row in 0..$output_info.height {
            $( let mut $precoli = $precols; )*
            for col in 0..$output_info.width {
                $(
                    #[allow(unused_variables)]
                    let [$r, $g, $b] = $frame.get((row * $output_info.width + col) as usize).unwrap().clone();
                )+
                $( let $px = row * $output_info.width + col; )?
                $action
            }
            $( $postcol )?
        }
        $( return $end; )?
        #[allow(unreachable_code)]
        $o
    }};
}

struct GroupCount<T: GroupCountValue> {
    stack: Vec<(T, u8)>,
    value: T,
    repeat: u8,
}

trait GroupCountValue: Copy + PartialEq {
    fn push(self, row: &mut Vec<u8>);
}

impl GroupCountValue for u8 {
    fn push(self, row: &mut Vec<u8>) {
        row.push(self);
    }
}

impl GroupCountValue for [u8; 3] {
    fn push(self, row: &mut Vec<u8>) {
        row.push(self[0]);
        row.push(self[1]);
        row.push(self[2]);
    }
}

impl<T: GroupCountValue> GroupCount<T> {
    fn new(default_value: T) -> GroupCount<T> {
        GroupCount {
            stack: Vec::new(),
            value: default_value,
            repeat: 0,
        }
    }

    fn append(&mut self, value: T) {
        if !self.value.eq(&value) || self.repeat == 255 {
            if self.repeat != 0 {
                self.stack.push((self.value, self.repeat));
            }
            self.value = value;
            self.repeat = 1;
        }
        else {
            self.repeat += 1;
        }
    }

    fn finalize(&mut self) -> Bytevec {
        if self.repeat != 0 {
            self.stack.push((self.value, self.repeat));
        }

        let mut row = Vec::new();
        for color in self.stack.iter() {
            row.push(color.1);
            color.0.push(&mut row);
        }
        row
    }
}

pub fn reduce_full_frame_raw(
    output_info: &OutputInfo,
    frame: &Frame,
) -> Bytevec {
    pixel_action!(1; output_info>o;
    |frame| [r g b] {
        o.extend(vec![r, g, b]);
    })
}

pub fn reduce_full_frame_rgb_count(
    output_info: &OutputInfo,
    frame: &Frame,
) -> Bytevec {
    pixel_action!(2; output_info > o
        rgbs = GroupCount::new([0, 0, 0]); {
            o.extend(rgbs.finalize());
        },
    |frame| [r g b] {
        rgbs.append([r, g, b]);
    })
}

pub fn reduce_full_frame_rgb_count_split(
    output_info: &OutputInfo,
    frame: &Frame,
) -> Bytevec {
    pixel_action!(3; output_info > o
        reds = GroupCount::new(0),
        greens = GroupCount::new(0),
        blues = GroupCount::new(0); {
            o.extend([&reds.finalize()[..], &greens.finalize()[..], &blues.finalize()[..]].concat());       
        },
    |frame| [r g b] {
        reds.append(r);
        greens.append(g);
        blues.append(b);
    })
}

pub fn reduce_partial_repeat(
    output_info: &OutputInfo,
    frame: &Frame,
    previous_frame: &Frame,
) -> Bytevec {
    let mut changed_pixels = Vec::<(u32, [u8; 3])>::new();
    pixel_action!(4; output_info & changed_pixels > o;
    |frame| [r g b]
    |previous_frame| [r2 g2 b2]
    px {
        if r != r2 || g != g2 || b != b2 {
            changed_pixels.push((px, [r, g, b]));
        }
    } => {
        for changed_pixel in changed_pixels {
            o.extend(&changed_pixel.0.to_be_bytes());
            o.extend(&changed_pixel.1);
        }
        o
    })
}

pub fn auto_reduce_frame(
    output_info: &OutputInfo,
    frame: &Frame,
    previous_frame: Option<&Frame>,
    force_full: bool,
) -> Bytevec {
    let (_rgb_count, _rgb_count_split) = (reduce_full_frame_rgb_count(&output_info, &frame), reduce_full_frame_rgb_count_split(&output_info, &frame));
    let mut compare: Vec<(usize, Box<dyn Fn() -> Bytevec>)> = vec![
        ((output_info.height * output_info.width * 3) as usize, Box::new(|| reduce_full_frame_raw(&output_info, &frame))),
        (_rgb_count.len(), Box::new(|| _rgb_count.clone())),
        (_rgb_count_split.len(), Box::new(|| _rgb_count_split.clone()))
    ];
    if let Some(previous) = previous_frame {
        if !force_full {
            let _partial_repeat = reduce_partial_repeat(&output_info, &frame, &previous);
            compare.push((_partial_repeat.len(), Box::new(move || _partial_repeat.clone())));
        }
    }
    compare.iter().min_by(|(s, _), (s2, _)| s.cmp(s2)).unwrap().1()
}
