# FreeType example in rust
This is a [FreeType](https://freetype.org) Simple text rendering, with kerning(untested), example in rust language.

Originally, I was making it to use in a game,<a href="footnote-1" id="ref-footnote-1">[1]</a> for font rendering with less ownership/borrowing problem. but after finding out that kerning with OpenType GPOS was not supported, I got fed up and just released it.

<span id="footnote-1"><a href="ref-footnote-1">[1]</a> I and my friends used sdl2-ttf. But, TTF feature of [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2) caused ownership/borrowing problems, driving me and the friends crazy.</span>

## (Korean) Rust로 작성한 FreeType 예시
[FreeType](https://freetype.org) 라이브러리를 이용해 간단한 텍스트를 렌더링하는 예시입니다. (테스트는 안해봤지만 커닝도 합니다.)

게임 내에서 sdl2-ttf를 이용해서 개발하고 있었는데, 어느 순간 소유권/대여 문제를 일으키면서 대규모 리팩토링를 하게 되더라고요. 그래서 좀 짜증나서 리팩토링을 덜해도 되는 방향으로 간단한 라이브러리를 만들려 했습니다. (구조를 보면 그런 의도를 느낄 수 있습니다. `clone`를 막해도 잘 작동하게 만든 점이라던가...)

근데 막상 만들고나서 테스트해보니 커닝이 안 되더라고요... 확인해보니 FreeType은 OpenType의 GPOS를 지원 안 해서 그렇다는 걸 깨달았습니다.

몇시간 헛짓거리 하는데 날리고 진빠졌는데 막상 작업한 걸 삭제하자니 아까워서 GitHub에 공개합니다.

## License
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <http://unlicense.org/>