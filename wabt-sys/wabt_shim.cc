#include "src/wast-lexer.h"
#include "src/wast-parser.h"
#include "src/error-handler.h"
#include "src/resolve-names.h"

extern "C" {

wabt::Result::Enum wabt_resolve_names_script(
    wabt::WastLexer* lexer,
    wabt::Script* script,
    wabt::ErrorHandlerBuffer* error_handler) {
  return ResolveNamesScript(lexer, script, error_handler);
}

}
