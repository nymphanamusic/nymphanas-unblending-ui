use std::num::NonZero;
use std::sync::{Arc, Mutex};
use std::thread::{available_parallelism, Scope};

pub fn parallel_for_2d<'scope, 'env, T>(
    width: usize,
    height: usize,
    target_concurrency: Option<usize>,
    scope: &'scope Scope<'scope, 'env>,
    function: T,
) where
    T: FnMut(usize, usize) + Send + Sync + 'scope,
{
    let function = Arc::new(Mutex::new(function));
    let n_threads = (width * height).min(
        target_concurrency.unwrap_or(
            available_parallelism()
                .unwrap_or(NonZero::new(4).unwrap())
                .into(),
        ),
    );
    (0..n_threads).for_each(|j| {
        let function = Arc::clone(&function);
        scope.spawn(move || {
            let n = width * height;

            let start_index = j * (n / n_threads);
            let end_index = if n_threads == j + 1 {
                n
            } else {
                (j + 1) * (n / n_threads)
            };

            (start_index..end_index).for_each(|k| {
                function.lock().unwrap()(k % width, k / width);
            });
        });
    });
}
