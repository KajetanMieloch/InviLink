/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/invilink.json`.
 */
export type Invilink = {
  "address": "DQCwvnVHUzUuv23P6FBJESBn3h6X7XGKtZZ6W9nrVfNW",
  "metadata": {
    "name": "invilink",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "activateEvent",
      "discriminator": [
        231,
        184,
        218,
        110,
        194,
        0,
        39,
        115
      ],
      "accounts": [
        {
          "name": "event",
          "writable": true
        },
        {
          "name": "organizer",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "addOrganizer",
      "discriminator": [
        142,
        52,
        252,
        155,
        155,
        95,
        29,
        215
      ],
      "accounts": [
        {
          "name": "organizersPool",
          "writable": true
        },
        {
          "name": "signer",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "newOrganizer",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "createEventSeating",
      "discriminator": [
        235,
        92,
        108,
        158,
        159,
        112,
        128,
        66
      ],
      "accounts": [
        {
          "name": "event",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "eventId"
              }
            ]
          }
        },
        {
          "name": "seatingMap",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  101,
                  97,
                  116,
                  105,
                  110,
                  103,
                  95,
                  109,
                  97,
                  112
                ]
              },
              {
                "kind": "arg",
                "path": "eventId"
              }
            ]
          }
        },
        {
          "name": "organizersPool",
          "writable": true
        },
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "organizer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "eventId",
          "type": "string"
        },
        {
          "name": "name",
          "type": "string"
        },
        {
          "name": "eventDate",
          "type": "i64"
        },
        {
          "name": "ticketPrice",
          "type": "u64"
        },
        {
          "name": "availableTickets",
          "type": "u64"
        }
      ]
    },
    {
      "name": "deactivateEvent",
      "discriminator": [
        222,
        84,
        182,
        86,
        46,
        110,
        215,
        19
      ],
      "accounts": [
        {
          "name": "event",
          "writable": true
        },
        {
          "name": "organizer",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "deleteEvent",
      "discriminator": [
        103,
        111,
        95,
        106,
        232,
        24,
        190,
        84
      ],
      "accounts": [
        {
          "name": "event",
          "writable": true
        },
        {
          "name": "registry",
          "writable": true
        },
        {
          "name": "organizer",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "emitSeatingMapDetails",
      "discriminator": [
        197,
        139,
        164,
        115,
        182,
        233,
        126,
        93
      ],
      "accounts": [
        {
          "name": "seatingMap"
        }
      ],
      "args": []
    },
    {
      "name": "initializeEventRegistry",
      "discriminator": [
        222,
        221,
        108,
        11,
        214,
        161,
        6,
        121
      ],
      "accounts": [
        {
          "name": "registry",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  101,
                  110,
                  116,
                  95,
                  114,
                  101,
                  103,
                  105,
                  115,
                  116,
                  114,
                  121
                ]
              }
            ]
          }
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "initializeOrganizersPool",
      "discriminator": [
        213,
        153,
        51,
        23,
        150,
        192,
        71,
        166
      ],
      "accounts": [
        {
          "name": "organizersPool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  114,
                  103,
                  97,
                  110,
                  105,
                  122,
                  101,
                  114,
                  115,
                  95,
                  112,
                  111,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "initializeSeating",
      "discriminator": [
        76,
        232,
        16,
        246,
        55,
        146,
        234,
        22
      ],
      "accounts": [
        {
          "name": "seatingMap",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  101,
                  97,
                  116,
                  105,
                  110,
                  103,
                  95,
                  109,
                  97,
                  112
                ]
              },
              {
                "kind": "arg",
                "path": "eventId"
              }
            ]
          }
        },
        {
          "name": "organizer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "eventId",
          "type": "string"
        }
      ]
    },
    {
      "name": "initializeSeatingSection",
      "discriminator": [
        151,
        223,
        44,
        246,
        213,
        70,
        7,
        65
      ],
      "accounts": [
        {
          "name": "seatingMap",
          "writable": true
        },
        {
          "name": "seatingSection",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  101,
                  97,
                  116,
                  105,
                  110,
                  103,
                  95,
                  115,
                  101,
                  99,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "event"
              },
              {
                "kind": "arg",
                "path": "sectionName"
              }
            ]
          }
        },
        {
          "name": "event"
        },
        {
          "name": "organizer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "sectionName",
          "type": "string"
        },
        {
          "name": "sectionType",
          "type": "u8"
        },
        {
          "name": "rows",
          "type": "u8"
        },
        {
          "name": "seatsPerRow",
          "type": "u8"
        },
        {
          "name": "ticketPrice",
          "type": "u64"
        }
      ]
    },
    {
      "name": "mintTicketNft",
      "discriminator": [
        212,
        78,
        142,
        4,
        188,
        28,
        203,
        17
      ],
      "accounts": [
        {
          "name": "event",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "arg",
                "path": "eventId"
              }
            ]
          }
        },
        {
          "name": "buyer",
          "writable": true,
          "signer": true
        },
        {
          "name": "seatingMap",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  101,
                  97,
                  116,
                  105,
                  110,
                  103,
                  95,
                  109,
                  97,
                  112
                ]
              },
              {
                "kind": "arg",
                "path": "eventId"
              }
            ]
          }
        },
        {
          "name": "seatingSection",
          "writable": true
        },
        {
          "name": "mint",
          "writable": true
        },
        {
          "name": "tokenAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "buyer"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
              },
              {
                "kind": "account",
                "path": "mint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "metadata",
          "writable": true
        },
        {
          "name": "masterAccount",
          "writable": true,
          "address": "4Wg5ZqjS3AktHzq34hK1T55aFNKSjBpmJ3PyRChpPNDh"
        },
        {
          "name": "organizerWallet",
          "writable": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "tokenMetadataProgram",
          "address": "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "eventId",
          "type": "string"
        },
        {
          "name": "eventName",
          "type": "string"
        },
        {
          "name": "sectionName",
          "type": "string"
        },
        {
          "name": "row",
          "type": "u8"
        },
        {
          "name": "seat",
          "type": "u8"
        }
      ]
    },
    {
      "name": "removeOrganizer",
      "discriminator": [
        64,
        187,
        72,
        87,
        252,
        241,
        195,
        60
      ],
      "accounts": [
        {
          "name": "organizersPool",
          "writable": true
        },
        {
          "name": "signer",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "organizerToRemove",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "removeSeatingSection",
      "discriminator": [
        26,
        199,
        35,
        22,
        4,
        211,
        10,
        86
      ],
      "accounts": [
        {
          "name": "seatingMap",
          "writable": true
        },
        {
          "name": "seatingSection",
          "docs": [
            "Konto sekcji, która ma zostać usunięta.",
            "Atrybut `close = organizer` spowoduje, że środki z tego konta zostaną przekazane organizatorowi przy zamykaniu."
          ],
          "writable": true
        },
        {
          "name": "event"
        },
        {
          "name": "organizer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "updateEvent",
      "discriminator": [
        70,
        108,
        211,
        125,
        171,
        176,
        25,
        217
      ],
      "accounts": [
        {
          "name": "event",
          "writable": true
        },
        {
          "name": "organizer",
          "signer": true
        }
      ],
      "args": [
        {
          "name": "newName",
          "type": {
            "option": "string"
          }
        },
        {
          "name": "newTicketPrice",
          "type": {
            "option": "u64"
          }
        },
        {
          "name": "newAvailableTickets",
          "type": {
            "option": "u64"
          }
        }
      ]
    },
    {
      "name": "updateEventSeatingType",
      "discriminator": [
        89,
        204,
        60,
        113,
        135,
        16,
        115,
        142
      ],
      "accounts": [
        {
          "name": "event",
          "writable": true
        },
        {
          "name": "newSeatingMap",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  101,
                  97,
                  116,
                  105,
                  110,
                  103,
                  95,
                  109,
                  97,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "event.event_id",
                "account": "eventNft"
              }
            ]
          }
        },
        {
          "name": "organizer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "newSeatingType",
          "type": "u8"
        }
      ]
    },
    {
      "name": "updateSeatingSection",
      "discriminator": [
        46,
        155,
        128,
        9,
        243,
        228,
        210,
        182
      ],
      "accounts": [
        {
          "name": "seatingMap",
          "writable": true
        },
        {
          "name": "seatingSection",
          "writable": true
        },
        {
          "name": "event"
        },
        {
          "name": "organizer",
          "writable": true,
          "signer": true
        }
      ],
      "args": [
        {
          "name": "newRows",
          "type": {
            "option": "u8"
          }
        },
        {
          "name": "newSeatsPerRow",
          "type": {
            "option": "u8"
          }
        },
        {
          "name": "newSectionType",
          "type": {
            "option": "u8"
          }
        },
        {
          "name": "newTicketPrice",
          "type": {
            "option": "u64"
          }
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "eventNft",
      "discriminator": [
        20,
        200,
        78,
        14,
        85,
        211,
        167,
        152
      ]
    },
    {
      "name": "eventRegistry",
      "discriminator": [
        134,
        46,
        144,
        236,
        160,
        28,
        35,
        60
      ]
    },
    {
      "name": "organizersPool",
      "discriminator": [
        213,
        169,
        121,
        137,
        55,
        255,
        214,
        37
      ]
    },
    {
      "name": "seatingMap",
      "discriminator": [
        30,
        248,
        137,
        8,
        84,
        233,
        139,
        229
      ]
    },
    {
      "name": "seatingSectionAccount",
      "discriminator": [
        222,
        195,
        116,
        215,
        221,
        155,
        23,
        252
      ]
    }
  ],
  "events": [
    {
      "name": "seatingMapDetails",
      "discriminator": [
        167,
        203,
        254,
        83,
        21,
        160,
        57,
        153
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "unauthorized",
      "msg": "Unauthorized operation."
    },
    {
      "code": 6001,
      "name": "alreadyRegistered",
      "msg": "This organizer is already registered."
    },
    {
      "code": 6002,
      "name": "organizerNotFound",
      "msg": "This organizer is not found in the list."
    },
    {
      "code": 6003,
      "name": "invalidSeatingType",
      "msg": "Invalid seating type."
    },
    {
      "code": 6004,
      "name": "seatNotReserved",
      "msg": "This seat was not reserved."
    },
    {
      "code": 6005,
      "name": "insufficientFunds",
      "msg": "Insufficient funds."
    },
    {
      "code": 6006,
      "name": "invalidTicket",
      "msg": "Invalid ticket ID."
    },
    {
      "code": 6007,
      "name": "ticketAlreadyUsed",
      "msg": "Ticket has already been used."
    },
    {
      "code": 6008,
      "name": "adminOnly",
      "msg": "Only the administrator can withdraw fees."
    },
    {
      "code": 6009,
      "name": "invalidSeating",
      "msg": "Invalid seating configuration."
    },
    {
      "code": 6010,
      "name": "seatAlreadyTaken",
      "msg": "This seat is already taken."
    },
    {
      "code": 6011,
      "name": "eventNotActive",
      "msg": "Event is not active."
    },
    {
      "code": 6012,
      "name": "eventIsActive",
      "msg": "Event is active and cannot be updated."
    },
    {
      "code": 6013,
      "name": "invalidTicketQuantity",
      "msg": "New available tickets cannot be less than the number of sold tickets."
    },
    {
      "code": 6014,
      "name": "cannotUpdateSeatingAfterSales",
      "msg": "Cannot update seating configuration after tickets have been sold."
    },
    {
      "code": 6015,
      "name": "registryFull",
      "msg": "Event registry is full."
    },
    {
      "code": 6016,
      "name": "cannotRemoveSectionWithTickets",
      "msg": "Cannot remove section: some tickets are sold or reserved."
    },
    {
      "code": 6017,
      "name": "cannotChangeSeatingType",
      "msg": "Cannot change event type because tickets have already been sold. Only allowed to change from numbered to mixed."
    },
    {
      "code": 6018,
      "name": "invalidEventId",
      "msg": "Invalid event ID."
    },
    {
      "code": 6019,
      "name": "eventCannotDeactivate",
      "msg": "Event cannot be deactivated because tickets have been sold."
    }
  ],
  "types": [
    {
      "name": "eventNft",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventId",
            "type": "string"
          },
          {
            "name": "organizer",
            "type": "pubkey"
          },
          {
            "name": "name",
            "type": "string"
          },
          {
            "name": "eventDate",
            "type": "i64"
          },
          {
            "name": "ticketPrice",
            "type": "u64"
          },
          {
            "name": "availableTickets",
            "type": "u64"
          },
          {
            "name": "soldTickets",
            "type": "u64"
          },
          {
            "name": "seatingType",
            "type": "u8"
          },
          {
            "name": "active",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "eventRegistry",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventCount",
            "type": "u32"
          },
          {
            "name": "events",
            "type": {
              "array": [
                "pubkey",
                10
              ]
            }
          }
        ]
      }
    },
    {
      "name": "organizersPool",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "organizers",
            "type": {
              "vec": "pubkey"
            }
          }
        ]
      }
    },
    {
      "name": "seatingMap",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventId",
            "type": "string"
          },
          {
            "name": "organizer",
            "type": "pubkey"
          },
          {
            "name": "active",
            "type": "bool"
          },
          {
            "name": "sections",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "totalSeats",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "seatingMapDetails",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventId",
            "type": "string"
          },
          {
            "name": "totalSeats",
            "type": "u64"
          },
          {
            "name": "sections",
            "type": {
              "vec": "pubkey"
            }
          }
        ]
      }
    },
    {
      "name": "seatingSectionAccount",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventId",
            "type": "string"
          },
          {
            "name": "sectionName",
            "type": "string"
          },
          {
            "name": "sectionType",
            "type": "u8"
          },
          {
            "name": "rows",
            "type": "u8"
          },
          {
            "name": "seatsPerRow",
            "type": "u8"
          },
          {
            "name": "ticketPrice",
            "type": "u64"
          },
          {
            "name": "seatStatus",
            "type": "bytes"
          }
        ]
      }
    }
  ]
};
