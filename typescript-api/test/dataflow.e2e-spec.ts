import { injectFunctionName } from "../src/common/common";

describe("test inject function name", () => {
  it("should wrap for one argument", function() {
    let fn = (v: string) => {
      return v.split(" ");
    };

    let result = injectFunctionName("processor", fn.toString());

    expect(result).toEqual("function processor (v)  {\n            return v.split(\" \");\n        }");
  });

  it("should wrap for two arguments", function() {
    let fn = (v: string, u: string) => {
      return v.concat(u);
    };

    let result = injectFunctionName("processor", fn.toString());

    expect(result).toEqual("function processor (v, u)  {\n            return v.concat(u);\n        }");
  });
});