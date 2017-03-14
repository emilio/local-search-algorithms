
const FFI_CACHE: any = {};

class Solution {
  constructor(public queenRows: Uint32Array,
              public score: number) {}
}

class AlgorithmConfig {
  constructor(public name: string,
              public extra_args: number[]) {}
}

class ASMInterface {
  constructor() {}

  solve(n: number,
        name: string,
        stepCallback?: (state: Uint32Array, score: number) => void
        ...args: number[]) : Uint32Array {
    name = "solve_n_queens_" + name;
    if (!FFI_CACHE[name]) {
      let arg_kinds = ['number', 'number'];
      for (let arg of args)
        arg_kinds.push('number');
      FFI_CACHE[name]=
        Module.cwrap(name, 'number', arg_kinds);
    }

    let asmCallback = 0;
    if (stepCallback) {
      asmCallback = Runtime.addFunction(function(ptr, len, score) {
        let state = new Uint32Array(len);
        for (let i = 0; i < len; ++i)
          state[i] = Module.getValue(ptr + i * 4, 'i32');
        console.log(state, score);
        stepCallback(state, score);
      });
    }

    let mem = Module._malloc((n + 1) * 4);

    let solutionScore =
      FFI_CACHE[name](n, mem, asmCallback, ...args);

    let resultLen = Module.getValue(mem, 'i32');
    let rows = new Uint32Array(resultLen);
    for (var i = 0; i < resultLen; ++i)
      rows[i] = Module.getValue(mem + (i + 1) * 4, 'i32');

    if (asmCallback)
      Runtime.removeFunction(asmCallback);
    Module._free(mem);

    return new Solution(rows, solutionScore);
  }
}

function waitABit() {
  const A_BIT: number = 200;
  return new Promise<void>(function(resolve, reject) {
    setTimeout(resolve, A_BIT);
  });
}

class Application {
  public asmInterface: ASMInterface;

  constructor(public grid: HTMLDivElement,
              public scoreBoard: HTMLElement,
              public numberChooser: HTMLInputElement,
              public algorithmChooser: HTMLSelectElement,
              public simulatedAnnealingInitialTemperature: HTMLInputElement,
              public simulatedAnnealingCoolingFactor: HTMLInputElement,
              public localBeamSearchStateCount: HTMLInputElement,
              public geneticGenerationSize: HTMLInputElement,
              public geneticElitismPercent: HTMLInputElement,
              public geneticCrossoverProbability: HTMLInputElement,
              public geneticMutationProbability: HTMLInputElement,
              public geneticGenerationCount: HTMLInputElement,
              public runButton: HTMLElement) {
    this.asmInterface = new ASMInterface();
  }

  run() {
    this.runButton.addEventListener('click', e => {
      this.runWithCurrentState();
    })
  }

  currentAlgorithm() : AlgorithmConfig {
    let name = this.algorithmChooser.options[this.algorithmChooser.selectedIndex].value;
    let args = [];

    function percent(input: HTMLInputElement) : number {
      return Math.max(0, Math.min(1, input.valueAsNumber / 100));
    }

    switch (name) {
      case "hill_climbing":
      case "constraint_propagation":
        break;
      case "simulated_annealing":
        args.push(this.simulatedAnnealingInitialTemperature.valueAsNumber)
        args.push(percent(this.simulatedAnnealingCoolingFactor));
        break;
      case "local_beam_search":
        args.push(this.localBeamSearchStateCount.valueAsNumber);
        break;
      case "genetic":
        args.push(this.geneticGenerationSize.valueAsNumber)
        args.push(percent(this.geneticElitismPercent));
        args.push(percent(this.geneticCrossoverProbability));
        args.push(percent(this.geneticMutationProbability));
        args.push(this.geneticGenerationCount.valueAsNumber);
        break;

      default:
        return null;
    }

    return new AlgorithmConfig(name, args);
  }

  async runWithCurrentState() {
    const count = this.numberChooser.valueAsNumber;
    this.grid.classList.add('no-solution');
    this.grid.innerHTML = "";
    this.scoreBoard.innerHTML = "";
    let algorithmConfig = this.currentAlgorithm();

    if (count < 0 || !algorithmConfig)
      return;
    this.grid.classList.remove('no-solution');
    let items: Array<HTMLDivElement> = new Array(count * count);
    for (let i = 0; i < count * count; ++i) {
      items[i] = document.createElement('div');
      const columnIsEven = (i % count) % 2 == 0;
      const rowIsEven = Math.floor(i / count) % 2 == 0;
      if (columnIsEven != rowIsEven)
        items[i].classList.add('black');
    }

    let percentage = count == 0 ? 0 : 100 / count;
    this.grid.style.gridTemplateColumns = 'repeat(' + count + ', ' + percentage + '%)';

    let fragment = document.createDocumentFragment();
    for (let item of items)
      fragment.appendChild(item);
    this.grid.appendChild(fragment);

    let scoreBoard = this.scoreBoard;

    // FIXME(emilio): This gets all the steps in memory just to avoid using
    // an iterator pattern from Rust.
    //
    // This effectively sucks, because the step count can be quite big. That
    // being said, I might not fix it if not needed.
    let steps = new Array<Solution>();

    this.asmInterface.solve(count, algorithmConfig.name, function(queens, score) {
      steps.push(new Solution(queens, score));
    }, ...algorithmConfig.extra_args);

    let latestQueens = null;
    for (step of steps) {
      if (latestQueens) {
        for (let i = 0; i < latestQueens.length; ++i)
          items[i + latestQueens[i] * count].classList.remove('queen');
      }

      let queens = step.queenRows;
      for (let i = 0; i < queens.length; ++i)
        items[i + queens[i] * count].classList.add('queen');

      scoreBoard.innerHTML = step.score;
      latestQueens = queens;
      await waitABit();
    }
  }
}
