use plotly::common::Mode;
use plotly::{Plot, Scatter, ImageFormat};

pub fn line_y(y: impl IntoIterator<Item=f32> + ExactSizeIterator) {
    let x = (0..y.len()).into_iter().map(|i| i as f32);
    line(x,y);
}

pub fn line(x: impl IntoIterator<Item=f32>, y: impl IntoIterator<Item=f32>) {
    // let trace1 = Scatter::new(vec![1, 2, 3, 4], vec![10, 15, 13, 17])
    //     .name("trace1")
    //     .mode(Mode::Markers);
    let trace2 = Scatter::new(x, y)
        .name("trace2")
        .mode(Mode::Lines);

    let mut plot = Plot::new();
    // plot.add_trace(trace1);
    plot.add_trace(trace2);

    // The following will save the plot in all available formats and show the plot.
    // plot.save("scatter", ImageFormat::PNG,  1024, 680, 1.0);
    plot.show();
}
