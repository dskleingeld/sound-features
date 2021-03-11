use rustfft::{FftPlanner, num_complex::Complex, Fft};
use rustfft::num_complex::Complex32;
use std::sync::Arc;

pub type FreqBand = std::ops::Range<usize>;

// // TODO enable when const generics are done
// const fn equal_bands<const N: usize>(start: usize, step: usize) -> [FreqBand; N] {
//     (0..N).into_iter()
//         .map(|i| i*step+start)
//         .map(|f| f..f+step)
//         .collect()
// }

#[derive(Default, Debug, Clone, Copy)]
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
        }
    }
    pub fn build(&mut self) -> Calculator<N, BINSIZE> {
        let fft = self.planner.plan_fft_forward(BINSIZE);
        let len = fft.get_inplace_scratch_len();
        let scratch= vec![Complex32::default(); len];

        Calculator {
            bands: self.bands.clone(),
            scratch,
            fft,
        }
    }
}

pub struct Calculator<const N: usize, const BINSIZE: usize> {
    bands: [IndexBand; N],
    fft: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex32>,
}

impl<const N: usize, const BINSIZE: usize> Calculator<N, BINSIZE> {
    pub fn process_slice(&mut self, samples: &[i16]) -> [f32; N] 
    {

        // TODO enable when supported by compiler
        // let samples: [Complex32; BINSIZE] = samples.into_iter()
        //     .map(|re| Complex32::new(*re as f32, 0f32))
        //     .collect();
        let mut buffer = [Complex32::new(0f32, 0f32); BINSIZE];
        for (int, complex) in samples.into_iter().zip(buffer.iter_mut()) {
            let float = *int as f32;
            *complex = Complex32::new(float, 0f32);
        }
        self.process_inner(buffer)
    }
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

        #[cfg(test)] {
            let y = fft.iter().map(|c| c.re);
            crate::plot::line_y(y);
        }
        
        let mut energies = [0f32; N];
        for (energy, IndexBand{start,end}) in energies.iter_mut().zip(&self.bands) {
            let band = &fft[*start..*end];
            *energy = band.iter()
                // take the absolute value of the real part and throw away the 
                // imaginary part (phase info) negative amplitudes are caused by out of 
                // phase waves, the abs() fixes that
                .map(|c| c.re.abs())
                .sum();
        }
        
        energies
    }
}

#[cfg(test)]
mod tests {
    use crate::plot;
    use super::*;

    #[test]
    fn sine() {
        use std::f32::consts::PI;
        use std::i16::MAX;
        let sample_rate = 44100;
        const BINSIZE: usize = 512;
        let freq1 = 500.;
        let freq2 = 200.;
        let freq3 = 120.;
        let amp = (MAX/4) as f32;

        let to_index = |f| f*(BINSIZE as f32)/(sample_rate as f32);
        dbg!(to_index(freq1));
        dbg!(to_index(freq2));
        dbg!(to_index(freq3));

        let mut samples = vec![0i16; 512];
        for (i,s) in samples.iter_mut().enumerate() {
            let time = (i as f32)/(sample_rate as f32);
            let mut v = 0f32;
            v += 0.3*amp*f32::sin(2.*PI*freq1*time);
            v += 0.5*amp*f32::sin(2.*PI*freq2*time);
            v += 0.2*amp*f32::sin(2.*PI*freq3*time);
            *s = v as i16;
        }
        
        let bands = [0..200, 200..400, 400..600, 600..800, 800..1000];
        let mut calc = Builder::<5, BINSIZE>::new(bands, sample_rate)
            .build();
        
        let chunks = samples.chunks(BINSIZE);
        let energies: Vec<_> = chunks
            .map(|chunk| calc.process_slice(chunk))
            .collect();

        dbg!(energies);
    }

    // #[test]
    // fn plot() {
    // }
}
