use iced_x86::{Decoder, DecoderOptions, Instruction, Register, MemorySize, OpKind};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct AssemblyToken {
    pub token_type: TokenType,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]

pub enum TokenType {
    Mnemonic,
    Register,
    Immediate,
    Memory,
    Prefix,
    Separator,
    Label,
}

pub struct AssemblyEncoder {
    vocabulary: HashMap<String, usize>,
    reverse_vocabulary: HashMap<usize, String>,
    next_token_id: usize,
}

impl AssemblyEncoder {
    pub fn new() -> Self {
        Self {
            vocabulary: HashMap::new(),
            reverse_vocabulary: HashMap::new(),
            next_token_id: 0,
        }
    }

    pub fn encode(&mut self, assembly: &str) -> Vec<AssemblyToken> {
        let bytes = assembly.as_bytes();
        let mut decoder = Decoder::with_ip(64, bytes, 0, DecoderOptions::NONE);
        let mut tokens = Vec::new();

        // Process each instruction
        let mut instruction = Instruction::default();
        while decoder.can_decode() {
            decoder.decode_out(&mut instruction);
            
            // Add mnemonic
            tokens.push(AssemblyToken {
                token_type: TokenType::Mnemonic,
                value: format!("{:?}", instruction.mnemonic()),
            });

            // Process operands
            for i in 0..instruction.op_count() {
                if i > 0 {
                    tokens.push(AssemblyToken {
                        token_type: TokenType::Separator,
                        value: ",".to_string(),
                    });
                }

                match instruction.op_kind(i) {
                    OpKind::Register => {
                        tokens.push(AssemblyToken {
                            token_type: TokenType::Register,
                            value: format!("{:?}", instruction.op_register(i)),
                        });
                    }
                    OpKind::Memory => {
                        self.encode_memory_operand(&instruction, i, &mut tokens);
                    }
                    OpKind::Immediate8 | OpKind::Immediate16 | OpKind::Immediate32 | OpKind::Immediate64 => {
                        tokens.push(AssemblyToken {
                            token_type: TokenType::Immediate,
                            value: format!("{:#x}", instruction.immediate(i)),
                        });
                    }
                    _ => {}
                }
            }
        }

        tokens
    }

    fn encode_memory_operand(&self, instruction: &Instruction, _operand_index: u32, tokens: &mut Vec<AssemblyToken>) {
        // Handle memory access size prefix
        let size = instruction.memory_size();
        if size != MemorySize::Unknown {
            tokens.push(AssemblyToken {
                token_type: TokenType::Prefix,
                value: format!("{:?}", size).to_lowercase(),
            });
        }

        tokens.push(AssemblyToken {
            token_type: TokenType::Memory,
            value: "[".to_string(),
        });

        // Base register
        if instruction.memory_base() != Register::None {
            tokens.push(AssemblyToken {
                token_type: TokenType::Register,
                value: format!("{:?}", instruction.memory_base()),
            });
        }

        // Index register
        if instruction.memory_index() != Register::None {
            if instruction.memory_base() != Register::None {
                tokens.push(AssemblyToken {
                    token_type: TokenType::Separator,
                    value: "+".to_string(),
                });
            }
            tokens.push(AssemblyToken {
                token_type: TokenType::Register,
                value: format!("{:?}", instruction.memory_index()),
            });

            // Scale
            let scale = instruction.memory_index_scale();
            if scale > 1 {
                tokens.push(AssemblyToken {
                    token_type: TokenType::Separator,
                    value: "*".to_string(),
                });
                tokens.push(AssemblyToken {
                    token_type: TokenType::Immediate,
                    value: scale.to_string(),
                });
            }
        }

        // Displacement
        let displacement = instruction.memory_displacement32();
        if displacement != 0 {
            if instruction.memory_base() != Register::None || instruction.memory_index() != Register::None {
                tokens.push(AssemblyToken {
                    token_type: TokenType::Separator,
                    value: "+".to_string(),
                });
            }
            tokens.push(AssemblyToken {
                token_type: TokenType::Immediate,
                value: format!("{:#x}", displacement),
            });
        }

        tokens.push(AssemblyToken {
            token_type: TokenType::Memory,
            value: "]".to_string(),
        });
    }

    pub fn get_vocabulary_size(&self) -> usize {
        self.vocabulary.len()
    }

    pub fn get_token_id(&mut self, token: &str) -> usize {
        if let Some(&id) = self.vocabulary.get(token) {
            id
        } else {
            let id = self.next_token_id;
            self.vocabulary.insert(token.to_string(), id);
            self.reverse_vocabulary.insert(id, token.to_string());
            self.next_token_id += 1;
            id
        }
    }

    pub fn get_token(&self, id: usize) -> Option<&str> {
        self.reverse_vocabulary.get(&id).map(|s| s.as_str())
    }
} 