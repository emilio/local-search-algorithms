
const FFI_CACHE: any = {};

class ASMInterface {
  constructor() {}

  solveHillClimbing(n: number, stepCallback?: (state: Uint32Array) => void) : Uint32Array {
    if (!FFI_CACHE.solve_n_queens_hill_climbing) {
      FFI_CACHE.solve_n_queens_hill_climbing =
        Module.cwrap('solve_n_queens_hill_climbing', 'number', ['number', 'number']);
    }

    let asmCallback = 0;
    if (stepCallback) {
      asmCallback = Runtime.addFunction(function(ptr, len) {
        let state = new Uint32Array(len);
        for (let i = 0; i < len; ++i)
          state[i] = Module.getValue(ptr + i * 4, 'i32');
        stepCallback(state);
      });
    }

    let mem = Module._malloc(n * 4);

    let solutionFound =
      FFI_CACHE.solve_n_queens_hill_climbing(n, mem, asmCallback);

    let ret = null;
    if (solutionFound) {
      ret = new Uint32Array(n);
      for (var i = 0; i < n; ++i)
        ret[i] = Module.getValue(mem + i * 4, 'i32');
    }

    if (asmCallback)
      Runtime.removeFunction(asmCallback);
    Module._free(mem);

    return ret;
  }
}

class Application {
  public asmInterface: ASMInterface;

  constructor(public grid: HTMLDivElement,
              public numberChooser: HTMLInputElement,
              public runButton: HTMLElement) {
    this.asmInterface = new ASMInterface();
  }

  run() {
    this.runButton.addEventListener('click', e => {
      const count = this.numberChooser.valueAsNumber;
      this.grid.classList.add('no-solution');
      this.grid.innerHTML = "";
      if (count < 0)
        return;
      const result = this.asmInterface.solveHillClimbing(count);
      if (!result)
        return;

      if (result.length != count)
        throw "Unexpected return value from Rust?";

      this.grid.classList.remove('no-solution');
      let array: Array<HTMLDivElement> = new Array(count * count);
      for (let i = 0; i < count * count; ++i) {
        array[i] = document.createElement('div');
        const columnIsEven = (i % count) % 2 == 0;
        const rowIsEven = Math.floor(i / count) % 2 == 0;
        if (columnIsEven != rowIsEven)
          array[i].classList.add('black');
      }

      for (let i = 0; i < count; ++i) {
        console.log(i);
        array[result[i]].classList.add('queen');
      }

      let percentage = count == 0 ? 0 : 100 / count;
      this.grid.style.gridTemplateColumns = 'repeat(' + count + ', ' + percentage + '%)';

      let fragment = document.createDocumentFragment();
      for (let item of array)
        fragment.appendChild(item);
      this.grid.appendChild(fragment);
    });
  }
}
