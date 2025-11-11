import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import ProtocolList from "../ProtocolList.vue";

describe("ProtocolList", () => {
  it("renders protocol names and metadata", () => {
    const wrapper = mount(ProtocolList, {
      props: {
        loading: false,
        protocols: [
          {
            id: "1",
            name: "Morning Stack",
            peptide_name: "BPC-157",
            notes: "Pre-workout",
            target_concentration_mg_ml: 2,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        ],
      },
    });

    expect(wrapper.text()).toContain("Morning Stack");
    expect(wrapper.text()).toContain("BPC-157");
  });
});
