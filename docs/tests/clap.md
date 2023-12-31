# TODO

* [ ] File freezing doesn't work
* [ ] 

# IMPORTANT

-> Test debug mode later, because it is incomplete

## Check clap flags

1. [x] Read from file and save to file
rad input_file.txt -o out_file.txt

2. [x] Read from file and print to stdout 
rad input_file.txt

3. [x] Read from standard input and print to file
printf 'text' | rad -o out_file.txt

4. [x] Read from stdin and print to stdout 
printf 'text' | rad 

5. [x] Print simple manual
rad --man 
rad --man ifdef

8. [x] Refer macro_syntax for further information
rad --comment
rad --comment any

11. [x] Permission argument is case insensitive
-a env                Give environment permission
-a cmd                Give syscmd permission
-a fin+fout           give both file read and file write permission
-A                    Give all permission. this is same with '-a env+cmd+fin+fout'
-w env                Give permission but warn when macro is used
-W                    Same with '-A' but for warning

13. [x] default is stderr
-e, --err <FILE>      1. Log error to <FILE>
-s, --silent <OPTION> 1. Suppress warnings default is all. All|Security|sanity|None
-l, --lenient         1. Disable strict mode. Print original if macro doesn't exist.
-p, --purge           1. Purge mode, print nothing if a macro doesn't exist.
    --assert          1. Enable assertion mode

17. [x] -D, --discard         1. Discard all output

[x] rad test --freeze --out frozen.r4f
[x] rad test --melt frozen.r4f 


17. [x] rad test --package --out bin.r4c + rad --script bin.r4c

22. [x] Print signature information into file
rad --signature sig.json

## LATER

16. you need to enable debug mode first to use other debug flags
-d, --debug           1. Start debug mode
    --log             1. Print all macro invocation logs
-i                    1. Start debug mode as interactive, this makes stdout unwrapped
    --diff            1. Show diff result between source and processed result
    --dryrun          1. Dry run macros

17. Other flags
-n                    1. Always use unix newline (default is '\r\n' in windows platform)

