# Machine Tree

Experimental project to find a proper API for building tree-structured persistent machines.

Inspired by React, I realized that tree is a very common structure for software. But despite a relatively big interest in React-inspired Rust projects, I cannot find any in the wild that concerns topics other than web-based GUI and WASM.

I have examined yew, sycamore, dioxus and all of those focus on WASM. I haven't looked at bastion.rs, which is inspired of erlang, but retains the tree-shape as well. I might have to look into it in the near future.

## Vision and use cases

In essence `machinetree` aims to make `react` ***without*** `react-dom` or `react-native`. It aims to enable programmers to construct tree-shaped machines and control how each of the submachines work with each other while actively managing the lifetime of the submachines based on the prewritten rules by the programmer, similar to how React spawns and destroyes nodes and tracks the relationships between. Think futures and async, but with easy to use porthole where programmers can also make a custom task scheduler so that they can customize the order of the "futures" execution.

Examples:
- DOM-based GUI
- Frame-buffer based GUI: Imagine each node of React representing a memoized framebuffer which don't update if there is no signals of change either from parent node or from the internal changes of the node.
- Multi-step 
- etc

## How it works

![How it Works!](./howitworks.jpg "How It Works")

## Status

- Does it work? Sort of.
- Does it lack any important feature? A lot. Context is what it is missing right now. Context is pretty important dependency injection mechanism. It enables a machine node to establish a dynamic value as a "dependency" of a set of sub machines inside a particular scope. Imagine injecting `fs::*` module and interchanging the actual implementation of `std::fs` and another implementation which saves to memory or network instead of disk AT RUNTIME.
- Is it usable in production? No. Don't even at this state.
- Is it safe? I used unsafe mem::transmute, though calculated, it will need peer review someday.
- Do you want contributors? Of course! If people are interested in this, feel free to play along or fork the project. Do invite me for coffee talk about it or Rust in general.