import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";

import { Actions } from "./actions";

function renderActions(voiceActionsAvailable = false): string {
  return renderToStaticMarkup(
    <Actions
      collapsed={false}
      microphoneMuted={false}
      navLayout="sidebar"
      onOpenAudioDevices={vi.fn()}
      onSetCollapsed={vi.fn()}
      onSetMicrophoneMuted={vi.fn()}
      onSetNavLayout={vi.fn()}
      onSetSoundMuted={vi.fn()}
      placement="sidebar"
      soundMuted={false}
      voiceActionsAvailable={voiceActionsAvailable}
    />,
  );
}

function buttonMarkup(markup: string, label: string): string {
  return markup.match(new RegExp(`<button[^>]*aria-label="${label}"[^>]*>`))?.[0] ?? "";
}

describe("profile actions", () => {
  it("disables stream and voice leave actions outside a call or voice channel", () => {
    const markup = renderActions();

    expect(buttonMarkup(markup, "Start stream")).toContain("disabled");
    expect(buttonMarkup(markup, "Leave voice")).toContain("disabled");
  });

  it("enables stream and voice leave actions inside a call or voice channel", () => {
    const markup = renderActions(true);

    expect(buttonMarkup(markup, "Start stream")).not.toContain("disabled");
    expect(buttonMarkup(markup, "Leave voice")).not.toContain("disabled");
  });
});
