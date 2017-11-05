# managed-alias

managed-alias is an alternative to the alias command. managed-alias allows you to maintain a list of aliases that you can modify on the file and is persitent across terminal sessions.

## Commamds
|               |                                                                                                                                                                            |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `set(s) <key> <value>` | Stores a key value pair                                                                                 |
| `delete(d) <key>`      | Delets the key value pair with the matching key                                                                     |
| `go(g)<key>`           | CDs into the directory stored for the provided key                                                          |
| `list(l)`              | Lists the stored key value pairs                                                                            |
| `run(r) <key>`         | Exectutes the value for the provided key. Values can be stored with placeholders `($0, $1, $2,` etc) and will be replaced with any additional arugments provided to the run subcommand |

## Installtion

### Linux 

CD into dist/linux and run `./install.sh`

Restart your terminal and run `ma --help` to verify things are working

### Mac(Install Script Coming)

The linux install script should work if you change .bashrc to .bash_profile 

You'll have to compile it yourself for now

### Windows(Install Script Coming)

Haven't tested this yet
