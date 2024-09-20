export default function load_blocks(Blockly) {
    const block_mixin = {
        init() {
            const n_initial_args = this.default_arg_names ? this.default_arg_names.length : 1
            this.inputCounter = n_initial_args // used to give each input a unique name
            for (let i = 0; i < n_initial_args; i++) {
                const input = this.appendValueInput(`arg_${i}`)
                if (i == 0) {
                    input.appendField(this.name)
                }
            }
            this.setHelpUrl(`https://github.com/ylxdzsw/sopipe/tree/master/components/${this.comp_name || this.name}`)
            this.tooltip && this.setTooltip(this.tooltip)
            this.setColour(this.color)
            this.sink_only || this.setNextStatement(true)
            this.source_only || this.setPreviousStatement(true)
        },

        mutationToDom() {
            const container = Blockly.utils.xml.createElement('mutation')
            const inputNames = this.inputList.map(input => input.name).join(',')
            container.setAttribute('inputNames', inputNames)
            container.setAttribute('inputCounter', this.inputCounter)
            return container
        },

        domToMutation(xmlElement) {
            const items = xmlElement.getAttribute('inputNames')
            if (items) {
                const inputNames = items.split(',')
                this.inputList = []
                inputNames.forEach((name) => this.appendValueInput(name))
                this.inputList[0].appendField(this.name)
            }
            this.inputCounter = parseInt(xmlElement.getAttribute('inputCounter'))
        },

        getIndexForNewInput(connection) {
            if (!connection.targetConnection) {
                return null
            }

            let connectionIndex
            for (let i = 0; i < this.inputList.length; i++) {
                if (this.inputList[i].connection == connection) {
                    connectionIndex = i
                }
            }

            if (connectionIndex == this.inputList.length - 1) {
                return this.inputList.length + 1
            }

            const nextInput = this.inputList[connectionIndex + 1]
            const nextConnection = nextInput && nextInput.connection.targetConnection
            if (nextConnection && !nextConnection.sourceBlock_.isInsertionMarker()) {
                return connectionIndex + 1
            }

            return null
        },

        onPendingConnection(connection) {
            const insertIndex = this.getIndexForNewInput(connection)
            if (insertIndex == null) {
                return
            }
            this.appendValueInput(`arg_${this.inputCounter++}`)
            this.moveNumberedInputBefore(this.inputList.length - 1, insertIndex)
        },

        finalizeConnections() {
            if (this.inputList.length > 1) {
                this.inputList
                    .slice(1)
                    .filter(input => !input.connection.targetConnection)
                    .map(input => this.removeInput(input.name))
            }
        },
    }

    const blocks = [{
        name: 'aead_encode',
        comp_name: 'aead',
        category: 'Encryption',
        default_arg_names: ['key']
    }, {
        name: 'aead_decode',
        comp_name: 'aead',
        category: 'Encryption',
        default_arg_names: ['key']
    }, {
        name: 'auth',
        category: 'Authentication',
        default_arg_names: ['key', 'salt']
    }, {
        name: 'balance',
        category: 'Trivia',
    }, {
        name: 'drop',
        category: 'Trivia',
    }, {
        name: 'echo',
        sink_only: true,
        category: 'Trivia',
    }, {
        name: 'exec',
        category: 'Trivia',
    }, {
        name: 'socks5_server',
        comp_name: 'socks5',
        category: 'Proxying',
    }, {
        name: 'vmess_client',
        comp_name: 'vmess',
        category: 'Proxying',
    }, {
        name: 'stdio',
        comp_name: 'stdio',
        category: 'Endpoints',
    }, {
        name: 'stdin',
        comp_name: 'stdio',
        source_only: true,
        category: 'Endpoints',
    }, {
        name: 'stdout',
        comp_name: 'stdio',
        sink_only: true,
        category: 'Endpoints',
    }, {
        name: 'tcp',
        category: 'Endpoints',
    }, {
        name: 'http2_client',
        comp_name: 'http2',
        sink_only: true,
        category: 'Endpoints',
    }, {
        name: 'tee',
        category: 'Trivia',
    }, {
        name: 'throttle',
        category: 'Trivia',
    }, {
        name: 'udp',
        category: 'Endpoints',
    }, {
        name: 'xor',
        category: 'Encryption',
        default_arg_names: ['key']
    }]

    let color_map = Object.create(null)
    let next_color = 40
    for (const block of blocks) {
        let color = color_map[block.category]
        if (!color) {
            color = next_color
            next_color += 60
            color_map[block.category] = color
        }

        Blockly.Blocks[block.name] = { ...block, ...block_mixin, color }
    }

    Blockly.Blocks['argument'] = {
        init() {
            this.setColour(20)
            this.setOutput(true)
            this.setTooltip('Set an argument.')
            this.appendDummyInput()
                .appendField(new Blockly.FieldTextInput("", s => s.replace(/\s/g, '')), "key") // TODO: real sanitizing
                .appendField("=")
                .appendField(new Blockly.FieldTextInput(""), "value")
        }
    }

    return blocks
}
