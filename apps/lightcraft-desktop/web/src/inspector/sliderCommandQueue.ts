export type SliderCommandParams = Record<string, unknown>;
export type SliderCommandRunner = (id: string, params?: SliderCommandParams) => Promise<unknown>;

type Deferred = {
  promise: Promise<void>;
  resolve: () => void;
  reject: (reason: unknown) => void;
};

type GenerationState = {
  pending: { params: SliderCommandParams; completion: Deferred } | null;
  pumpScheduled: boolean;
};

function deferred(): Deferred {
  let resolve!: () => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<void>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

/** Serializes interaction commands while collapsing unsent slider updates. */
export class SliderCommandQueue {
  private tail: Promise<void> = Promise.resolve();
  private activeGeneration: GenerationState | null = null;
  private readonly run: SliderCommandRunner;

  public constructor(run: SliderCommandRunner) {
    this.run = run;
  }

  public command(id: string, params: SliderCommandParams = {}): Promise<unknown> {
    const observed = this.tail.then(() => this.run(id, params));
    this.tail = observed.then(() => undefined, () => undefined);
    return observed;
  }

  public begin(label: string): Promise<unknown> {
    this.activeGeneration = { pending: null, pumpScheduled: false };
    return this.command('develop.beginInteraction', { label });
  }

  public set(control: string, value: number): Promise<void> {
    const generation = this.activeGeneration ?? { pending: null, pumpScheduled: false };
    this.activeGeneration = generation;
    const pending = generation.pending;
    if (pending) {
      pending.params = { control, value };
      return pending.completion.promise;
    }
    const completion = deferred();
    generation.pending = { params: { control, value }, completion };
    this.queueSetPump(generation);
    return completion.promise;
  }

  public end(): Promise<unknown> {
    if (this.activeGeneration) {
      this.queueSetPump(this.activeGeneration);
      this.activeGeneration = null;
    }
    return this.command('develop.endInteraction', {});
  }

  public cancel(): Promise<unknown> {
    if (this.activeGeneration) {
      const pending = this.activeGeneration.pending;
      if (pending) {
        pending.completion.resolve();
        this.activeGeneration.pending = null;
      }
      this.activeGeneration = null;
    }
    return this.command('develop.cancelInteraction', {});
  }

  private queueSetPump(generation: GenerationState): void {
    if (generation.pumpScheduled || !generation.pending) return;
    generation.pumpScheduled = true;
    const pump = this.tail.then(async () => {
      try {
        while (generation.pending) {
          const current = generation.pending;
          generation.pending = null;
          try {
            await this.run('develop.set', current.params);
            current.completion.resolve();
          } catch (reason: unknown) {
            current.completion.reject(reason);
          }
        }
      } finally {
        // Mark pump idle before its promise settles so a value arriving just
        // after this batch gets a task before end/cancel is enqueued.
        generation.pumpScheduled = false;
      }
    });
    this.tail = pump.then(() => undefined, () => undefined);
  }
}
