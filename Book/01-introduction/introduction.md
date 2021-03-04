# Entity Component Scrapyard: Introduction

Hiiii person reading this. 

EC-Scrapyard is a tutorial walkthrough thingy where we're going to write an ECS from scratch :3

This book is written with the assumption that the reader knows what an ECS is but not necessarily how it works or how to go about writing one. If
you dont know what an ECS is there's a good writeup [here](https://www.google.com)

A few caveats before we start:
  - We wont be using any dependencies, we will be using rust's standard library though :")
  - We will only use safe code so ``#![forbid(unsafe_code)]`` will be placed at the top of ``lib.rs``
  - We wont touch on the Systems part of ECS because I personally feel that's better left to engines and users of the ECS to decide how best to structure the program

A lot of the ECS' in the rust ecosystem use something called 'archetypes' so we'll be writing an archetype based ECS in this book. If you dont know what archetypes are don't worry they'll be explained later on in the book when we get around to implementing them :) If you're impatient and want to look at it now you can go ahead and do that :P [the section for that is here](./archetype-explanation)