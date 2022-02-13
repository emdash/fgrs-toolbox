# Overview

This crate is a 100% safe rust implementation of a family of
Functional Graph Rewriting algorithms (FGRS).

It is a translation into safe Rust of the ideas presented in:
[FPPGR](https://clean.cs.ru.nl/Functional_Programming_and_Parallel_Graph_Rewriting).

My motivation was to to understand the foundations of modern
functional programming systems, as well as advanced use of traits and
lifetimes in Rust.

Rules:
- Use only stable Rust (2021 edition at present).
- No unstable features.
- No unsafe code(*).
- No dependencies.
- Allow for 100% static dispatch.

Goals:
- Focus on runtime reduction, rather than compilation.
- Find the limits and sharp edges in Rust's type system and work around them.
- Create pluggable algorithms and data structures, using Rust's trait system.
- Provide default, efficient implementations for the most common cases.
- Benchmark the performance of different combinations of these
  elements.
- Analyize the quality of the code generated by the rust compiler.
- "Bring your own types" -- you can use the general implementationss
  to get started, substituting in your own optimized types as and when
  you need.

(*) Note: at the time of this writing, this rule prevents representing
graphs with pointers or direct references. I have concentrated on
other means of representing graphs.

# Bottom-up Beta Reduction

This crate startes as an attempt to implement DAG-based beta-reduction
algorithm described in [Shivers & Wand
2010](https://www.ccs.neu.edu/home/wand/papers/shivers-wand-10.pdf)

I lost the thread trying to translate their implementation from
Standard ML to rust, since it relied on both linked lists and in-place
mutation of the term graph. Basically, it wasn't going to be possible
to express their code directly in Rust without breaking my rules.

I didn't understand the algorithm well enough to translate it into
safe rust.

BUBR is now a narrow slice of the broader scope of this crate. I may
revisit it as and when something like GhostCell becomes part of the
standard library. Or, I may try a C++ translation. In any case, there
is more to FGRS than beta reduction.
