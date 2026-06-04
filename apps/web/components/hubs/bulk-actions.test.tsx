import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";

import { BulkActions } from "./bulk-actions";

describe("BulkActions", () => {
  it("uses shared button icon sizing without local SVG classes", () => {
    const markup = renderToStaticMarkup(
      <BulkActions
        busy={false}
        destructiveLabel="Remove"
        onDestructive={vi.fn()}
        onDone={vi.fn()}
        onMute={vi.fn()}
        onPin={vi.fn()}
        onUnmute={vi.fn()}
        onUnpin={vi.fn()}
        selectedCount={2}
      />,
    );

    expect(markup).toContain(">Pin<");
    expect(markup).toContain(">Mute<");
    expect(markup).not.toContain('class="icon"');
  });
});
