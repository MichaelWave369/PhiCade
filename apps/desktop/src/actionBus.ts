export type ActionSource =
  | { kind: "human"; seat: number }
  | { kind: "replay" }
  | { kind: "script"; name: string }
  | { kind: "phi-bot"; agentId: string; seat: number };

export type SystemCommand = "pause" | "reset" | "save-state" | "load-state" | "rewind";

export type GameAction =
  | { kind: "button"; button: string; pressed: boolean }
  | { kind: "axis"; axis: string; value: number }
  | { kind: "system"; command: SystemCommand; slot?: number | null };

export interface ActionEnvelope {
  sequence: number;
  frame: number;
  source: ActionSource;
  action: GameAction;
}

export class ActionBus {
  #sequence = 0;
  #queue: ActionEnvelope[] = [];

  publish(frame: number, source: ActionSource, action: GameAction): ActionEnvelope {
    const envelope = {
      sequence: this.#sequence++,
      frame,
      source,
      action,
    } satisfies ActionEnvelope;

    this.#queue.push(envelope);
    return envelope;
  }

  drainThrough(frame: number): ActionEnvelope[] {
    const split = this.#queue.findIndex((entry) => entry.frame > frame);
    if (split === -1) {
      return this.#queue.splice(0);
    }
    return this.#queue.splice(0, split);
  }

  get pending(): number {
    return this.#queue.length;
  }
}
