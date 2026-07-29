/*
 * Copyright (c) Radzivon Bartoshyk, 10/2024. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without modification,
 * are permitted provided that the following conditions are met:
 *
 * 1.  Redistributions of source code must retain the above copyright notice, this
 * list of conditions and the following disclaimer.
 *
 * 2.  Redistributions in binary form must reproduce the above copyright notice,
 * this list of conditions and the following disclaimer in the documentation
 * and/or other materials provided with the distribution.
 *
 * 3.  Neither the name of the copyright holder nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
use std::sync::Mutex;

pub(crate) fn for_each_chunk_with_scratch<T, F>(
    dst: &mut [T],
    chunk: usize,
    scratch_len: usize,
    work: F,
) where
    T: Default + Clone + Send,
    F: Fn(usize, &mut [T], &mut [T]) + Send + Sync,
{
    assert_ne!(chunk, 0, "Chunk size must never be zero");

    let chunks = dst.len() / chunk;
    if chunks == 0 {
        return;
    }

    let workers = rayon::current_num_threads().max(1).min(chunks);
    let queue = Mutex::new(dst.chunks_exact_mut(chunk).enumerate());

    rayon::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|_| {
                let mut scratch = vec![T::default(); scratch_len];

                loop {
                    // Bind the item first so the guard is released before `work` runs,
                    // otherwise the lock would serialize the workers.
                    let item = queue
                        .lock()
                        .expect("Scratch queue has been poisoned")
                        .next();

                    match item {
                        Some((index, dst_chunk)) => work(index, dst_chunk, &mut scratch),
                        None => break,
                    }
                }
            });
        }
    });
}
