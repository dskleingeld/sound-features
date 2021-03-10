#![feature(array_windows)]

use rodio::{Decoder, Source};
use itertools::Itertools;
use std::io::BufReader;
use std::fs::File;
use std::cmp::Ordering;

use audio_feature::band_energy;

fn argmax<'a>(arr: impl IntoIterator<Item=&'a f32>) -> usize {
    arr.into_iter()
        .enumerate()
        .reduce(|a,b| {
            if a.1 > b.1 {a}
            else {b}
        }).map(|(i,_)| i)
        .unwrap_or(0)
}

struct Bands<'a, const N: usize>(&'a [f32;N]);
impl<const N: usize> Bands<'_, N> {
    fn cmp(&self, other: &Self) -> Ordering {
        argmax(self.0).cmp(&argmax(other.0))
    }
}

fn energies_to_code<const N: usize>(energies: Vec<[f32;N]>) -> String {
    let mut code = String::from("*");
    for [previous, current] in energies.array_windows::<2>() {
        let current = Bands(current);
        let previous = Bands(previous);

        match current.cmp(&previous) {
            Ordering::Less => code.push('d'),
            Ordering::Greater => code.push('u'),
            Ordering::Equal => code.push('r'),
        }
    }
    code
}

#[derive(Debug, structopt::StructOpt)]
#[structopt(name = "parsons code", about = "Codes a signal as a sequence of pitch tendencies, like a Parsons code")]
struct Args {
    /// path to an audio file in wav, flac, vorbis or mp3 format
    #[structopt(parse(from_os_str))]
    input: std::path::PathBuf,
}

#[paw::main]
fn main(args: Args) {
    let f = File::open(args.input).unwrap();
    let reader = BufReader::new(f);
    let decoder = Decoder::new(reader).unwrap();
    let sample_rate = decoder.sample_rate();

    let bands = [0..1000, 1..2000, 
        1000..2000, 2000..3000, 
        3000..4000, 5000..6000];
    type Builder = band_energy::Builder::<6, 512>;
    let mut eng = Builder::new(bands, sample_rate)
        .build();

    let chunks = &decoder.chunks(512);
    let energies: Vec<_> = chunks.into_iter()
        .map(|chunk| eng.process(chunk))
        .collect();

    let n = sample_rate/512/2;
    let energies: Vec<_> = energies.chunks(n as usize)
        .map(|c| {
            let mut bands_sum = [0f32; 6];
            for b in c {
                for (v,sum) in b.iter().zip(&mut bands_sum) {
                    *sum += v;
                }
            }
            bands_sum
        }).collect();

    let code = energies_to_code(energies);
    println!("pseudo Parsons code: {}", code);
}
