use batdetect_core::{FrameParams, residual_and_threshold, temporal_median};
use numpy::{PyArray1, PyArrayDyn, PyArrayMethods, PyReadonlyArrayDyn, PyUntypedArrayMethods};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn same_shape(arrays: &[PyReadonlyArrayDyn<'_, f32>]) -> PyResult<Vec<usize>> {
    let shape = arrays.first().ok_or_else(|| PyValueError::new_err("needs at least one frame"))?.shape().to_vec();
    if arrays.iter().any(|a| a.shape() != shape.as_slice()) {
        return Err(PyValueError::new_err("frames differ in shape"));
    }
    Ok(shape)
}

fn slices<'a>(arrays: &'a [PyReadonlyArrayDyn<'_, f32>]) -> PyResult<Vec<&'a [f32]>> {
    arrays.iter().map(|a| a.as_slice().map_err(|_| PyValueError::new_err("frames must be contiguous"))).collect()
}

#[pyfunction]
#[pyo3(signature = (frames, parallel=true))]
fn temporal_median_f32<'py>(
    py: Python<'py>,
    frames: Vec<PyReadonlyArrayDyn<'py, f32>>,
    parallel: bool,
) -> PyResult<Bound<'py, PyArrayDyn<f64>>> {
    let shape = same_shape(&frames)?;
    let views = slices(&frames)?;
    let mut out = vec![0f64; views[0].len()];
    py.detach(|| temporal_median(&views, &mut out, parallel));
    PyArray1::from_vec(py, out).reshape(shape)
}

#[pyfunction]
#[pyo3(signature = (samples, current, mask, gain_offset, gains, threshold, noise_factor, use_noise, parallel=true))]
#[allow(clippy::too_many_arguments)]
fn residual_threshold<'py>(
    py: Python<'py>,
    samples: Vec<PyReadonlyArrayDyn<'py, f32>>,
    current: PyReadonlyArrayDyn<'py, f32>,
    mask: PyReadonlyArrayDyn<'py, bool>,
    gain_offset: f64,
    gains: Vec<f64>,
    threshold: f64,
    noise_factor: f64,
    use_noise: bool,
    parallel: bool,
) -> PyResult<(Bound<'py, PyArrayDyn<f64>>, Option<Bound<'py, PyArrayDyn<f64>>>)> {
    let shape = same_shape(&samples)?;
    if current.shape() != shape.as_slice() || mask.shape() != shape.as_slice() {
        return Err(PyValueError::new_err("current frame or mask differs in shape"));
    }
    let views = slices(&samples)?;
    let cur = current.as_slice().map_err(|_| PyValueError::new_err("current must be contiguous"))?;
    let m = mask.as_slice().map_err(|_| PyValueError::new_err("mask must be contiguous"))?;
    let n = cur.len();
    let (mut residual, mut thresholds) = (vec![0f64; n], vec![0f64; n]);
    let params = FrameParams { gain_offset, gains: &gains, threshold, noise_factor, use_noise };
    let any = py.detach(|| residual_and_threshold(&views, cur, m, &params, &mut residual, &mut thresholds, parallel));
    let residual = PyArray1::from_vec(py, residual).reshape(shape.clone())?;
    let map = if any { Some(PyArray1::from_vec(py, thresholds).reshape(shape)?) } else { None };
    Ok((residual, map))
}

#[pymodule]
fn batdetect_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(temporal_median_f32, m)?)?;
    m.add_function(wrap_pyfunction!(residual_threshold, m)?)?;
    m.add_function(wrap_pyfunction!(stages::resize_area_bgr, m)?)?;
    m.add_function(wrap_pyfunction!(stages::bgr_to_gray, m)?)?;
    m.add_function(wrap_pyfunction!(stages::gaussian_blur_f32, m)?)?;
    m.add_function(wrap_pyfunction!(stages::mean_f32, m)?)?;
    m.add_function(wrap_pyfunction!(stages::morph_close, m)?)?;
    m.add_function(wrap_pyfunction!(stages::components, m)?)?;
    m.add_function(wrap_pyfunction!(stages::find_blobs, m)?)?;
    m.add_function(wrap_pyfunction!(linker_padding, m)?)?;
    Ok(())
}

mod stages {
    use batdetect_core::frame;
    use numpy::{PyArray1, PyArrayDyn, PyArrayMethods, PyReadonlyArrayDyn, PyUntypedArrayMethods};
    use pyo3::prelude::*;

    fn dims(a: &PyReadonlyArrayDyn<'_, impl numpy::Element>) -> (usize, usize) {
        (a.shape()[1], a.shape()[0])
    }

    #[pyfunction]
    pub fn resize_area_bgr<'py>(
        py: Python<'py>,
        src: PyReadonlyArrayDyn<'py, u8>,
        dw: usize,
        dh: usize,
        fused: bool,
    ) -> PyResult<Bound<'py, PyArrayDyn<u8>>> {
        let (w, h) = dims(&src);
        let out = frame::resize_area_bgr(src.as_slice()?, w, h, dw, dh, fused);
        PyArray1::from_vec(py, out).reshape(vec![dh, dw, 3])
    }

    #[pyfunction]
    pub fn bgr_to_gray<'py>(py: Python<'py>, src: PyReadonlyArrayDyn<'py, u8>) -> PyResult<Bound<'py, PyArrayDyn<u8>>> {
        let (w, h) = dims(&src);
        PyArray1::from_vec(py, frame::bgr_to_gray(src.as_slice()?)).reshape(vec![h, w])
    }

    #[pyfunction]
    pub fn gaussian_blur_f32<'py>(
        py: Python<'py>,
        src: PyReadonlyArrayDyn<'py, u8>,
        kernel: Vec<f32>,
    ) -> PyResult<Bound<'py, PyArrayDyn<f32>>> {
        let (w, h) = dims(&src);
        PyArray1::from_vec(py, frame::gaussian_blur_f32(src.as_slice()?, w, h, &kernel)).reshape(vec![h, w])
    }

    #[pyfunction]
    pub fn mean_f32(src: PyReadonlyArrayDyn<'_, f32>) -> PyResult<f64> {
        Ok(frame::mean_f32(src.as_slice()?))
    }

    #[pyfunction]
    pub fn morph_close<'py>(
        py: Python<'py>,
        src: PyReadonlyArrayDyn<'py, u8>,
        size: usize,
    ) -> PyResult<Bound<'py, PyArrayDyn<u8>>> {
        let (w, h) = dims(&src);
        PyArray1::from_vec(py, frame::morph_close(src.as_slice()?, w, h, size)).reshape(vec![h, w])
    }

    #[pyfunction]
    #[allow(clippy::type_complexity)]
    pub fn components<'py>(
        py: Python<'py>,
        src: PyReadonlyArrayDyn<'py, u8>,
    ) -> PyResult<(usize, Bound<'py, PyArrayDyn<i32>>, Vec<(i64, i64, i64, i64, i64)>, Vec<(f64, f64)>)> {
        let (w, h) = dims(&src);
        let c = frame::components(src.as_slice()?, w, h);
        let stats = (0..c.count).map(|l| (c.left[l], c.top[l], c.width[l], c.height[l], c.area[l])).collect();
        let centroids = (0..c.count).map(|l| (c.cx[l], c.cy[l])).collect();
        Ok((c.count, PyArray1::from_vec(py, c.labels).reshape(vec![h, w])?, stats, centroids))
    }

    #[pyfunction]
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    pub fn find_blobs(
        residual: PyReadonlyArrayDyn<'_, f64>,
        thresholds: PyReadonlyArrayDyn<'_, f64>,
        min_area: f64,
        max_area: f64,
        merge_radius: f64,
        scale: f64,
    ) -> PyResult<Vec<(f64, f64, f64, f64, f64, f64, f64, f64)>> {
        let (w, h) = dims(&residual);
        let cfg = frame::BlobConfig { min_area, max_area, merge_radius };
        let blobs = frame::find_blobs_sparse(residual.as_slice()?, thresholds.as_slice()?, w, h, &cfg, scale);
        Ok(blobs.into_iter().map(|b| (b.x, b.y, b.left, b.top, b.width, b.height, b.area, b.amplitude)).collect())
    }
}

unsafe extern "C" {
    fn getpagesize() -> i32;
}

/// Imports one more symbol through the GOT so that the indirect symbol table keeps an even length:
/// ld-27037 does not pad it, and macOS 27 dyld refuses a string table that is not 8-byte aligned.
#[pyfunction]
fn linker_padding() -> usize {
    std::hint::black_box(getpagesize as unsafe extern "C" fn() -> i32) as usize
}
