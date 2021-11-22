export default function codegen(workspace, compact) {
    let result = ""

    function gen(head_block) {
        if (!head_block.name) { // all componenets have name. Other blocks hopefully not.
            return
        }

        let next = null
        let args = []
        for (const child of head_block.childBlocks_) {
            if (child.name) {
                next = child
                continue
            }

            args.push(gen_argument(child))
        }

        args = args.filter(x => x)

        result += head_block.name
        if (args.length) {
            result += `(${args.join(compact ? "," : ", ")})`
        }

        if (next) {
            result += compact ? "=>" : " =>\n"
            gen(next)
        }
    }

    function gen_argument(argument_block) {
        const [key_field, _, value_field] = argument_block.inputList[0].fieldRow
        const key = key_field.value_.trim()
        let value = value_field.value_.trim() // heuristic: if the value is quoted, it is string. Otherwise, if it is int, it is int; if it is empty, it is key-only; otherwise, treat as string and add quote

        if (value.charAt(0) != "\"" || value.charAt(value.length - 1) != "\"") {
            if (!value.length) { // empty, key-only argument
                if (!key.length) { // nothing, ignore this argument
                    return
                } else { // key-only
                    return key
                }
            }

            let int_value = parseInt(value)
            if (!isNaN(int_value)) { // int value
                value = '' + int_value
            } else { // unquoted string value
                value = `"${value}"`
            }
        }

        if (!key.length) {
            return value
        } else {
            return compact ? `${key}=${value}` : `${key} = ${value}`
        }
    }

    for (const top_block of workspace.getTopBlocks()) {
        gen(top_block)
        result += compact ? " " : "\n"
    }

    return result
}
