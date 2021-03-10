use rustfft::{FftPlanner, num_complex::Complex, Fft};
use rustfft::num_complex::Complex32;
use std::sync::Arc;

pub type FreqBand = std::ops::Range<usize>;

#[derive(Default, Clone, Copy)]
pub struct IndexBand {
    start: usize,
    end: usize,
}

impl IndexBand {
    // TODO error handeling, start and end can not be larger
    // then binsize
    fn from(band: &FreqBand, sample_rate: u32, binsize: usize) -> Self {
        let to_index = |f| f*binsize/(sample_rate as usize);
        Self {
            start: to_index(band.start),
            end: to_index(band.end),
        }
    }
}

pub struct Builder<const N: usize, const BINSIZE: usize> {
    bands: [IndexBand; N],
    planner: FftPlanner<f32>,
    sample_rate: u32,
}

impl<const N: usize, const BINSIZE: usize> Builder<N, BINSIZE> {
    pub fn new(freq_bands: [FreqBand; N], sample_rate: u32) -> Self { //TODO get binsize in via const new
        let mut bands = [IndexBand::default(); N]; // impossible no Copy allowed on range 
        for (band, freq_band) in bands.iter_mut().zip(freq_bands.iter()) {
            *band = IndexBand::from(freq_band, sample_rate, BINSIZE);
        }

        Self {
            bands,
            planner: FftPlanner::new(),
            sample_rate,
        }
    }
    pub fn build(&mut self) -> Calculator<N, BINSIZE> {
        let fft = self.planner.plan_fft_forward(BINSIZE);
        let len = fft.get_inplace_scratch_len();
        let scratch= vec![Complex32::default(); len];

        Calculator {
            sample_rate: self.sample_rate,
            bands: self.bands.clone(),
            scratch,
            fft,
        }
    }
}

pub struct Calculator<const N: usize, const BINSIZE: usize> {
    sample_rate: u32,
    bands: [IndexBand; N],
    fft: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex32>,
}

impl<const N: usize, const BINSIZE: usize> Calculator<N, BINSIZE> {
    pub fn process<'a,T>(&mut self, samples: T) -> [f32; N] 
        where
            T: IntoIterator<Item=i16>,
    {

        // TODO enable when supported by compiler
        // let samples: [Complex32; BINSIZE] = samples.into_iter()
        //     .map(|re| Complex32::new(*re as f32, 0f32))
        //     .collect();
        let mut buffer = [Complex32::new(0f32, 0f32); BINSIZE];
        for (int, complex) in samples.into_iter().zip(buffer.iter_mut()) {
            let float = int as f32;
            *complex = Complex32::new(float, 0f32);
        }
        self.process_inner(buffer)
    }
    fn process_inner(&mut self, mut buffer: [Complex32; BINSIZE]) -> [f32; N] {
        self.fft.process_with_scratch(&mut buffer, &mut self.scratch);
        let fft = buffer;
        
        let mut energies = [0f32; N];
        for (energy, IndexBand{start,end}) in energies.iter_mut().zip(&self.bands) {
            let band = &fft[*start..*end];
            *energy = band.iter()
                .map(|c| c.re)
                .sum();
        }
        
        energies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine() {
        use std::f32::consts::PI;
        use std::i16::MAX;
        let mut samples = vec![0i16; 512];
        let sample_rate = 100;
        const N: usize = 1;
        const BINSIZE: usize = 512;
        let test_freq = 4.;

        for (i,s) in samples.iter_mut().enumerate() {
            let time = (i as f32)/(sample_rate as f32);
            let v = f32::sin(2.*PI*test_freq*time);
            *s = ((MAX/2) as f32 * v) as i16;
        }
        
        let bands = [0..100];
        let energies = Builder::<N, BINSIZE>::new(bands, sample_rate)
            .build().process(samples);

        dbg!(energies);
    }
}
