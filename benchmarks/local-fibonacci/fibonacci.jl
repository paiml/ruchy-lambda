#!/usr/bin/env julia
# Julia fibonacci(35) benchmark
# Expected: ~25ms after JIT warmup (matches Rust performance)
# Runtime: ~200MB (Julia + LLVM)

function fibonacci(n::Int)::Int
    if n <= 1
        return n
    else
        return fibonacci(n - 1) + fibonacci(n - 2)
    end
end

function main()
    result = fibonacci(35)
    # Silent for benchmarking
end

main()
