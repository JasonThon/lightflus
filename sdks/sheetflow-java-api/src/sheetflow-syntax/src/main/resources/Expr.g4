grammar Expr;

expr: valueExpr;

valueExpr:
    nestedValueExpr
    | conditionExpr
    ;

nestedValueExpr:
    nestedValueExpr op=(MUL|DIV) nestedValueExpr    # mulDiv
    | nestedValueExpr op=(ADD|SUB) nestedValueExpr  # addSub
    | sequenceRef                                   # getByIndex
    | constant                                      # getByConst
    | valueFunc                                     # function
    | transtion                                     # transtioner
    | conditionFunc                                 # getByConditionFun
    | '(' nestedValueExpr ')'                       # parens
    ;

conditionExpr:
    '(' conditionExpr ')'                                       # prioritisedCondition
    | nestedValueExpr op=(LT|GT|EQ|NEQ|LTE|GTE) nestedValueExpr # checkCondition
    | conditionExpr op=AND conditionExpr                        # checkNestedCondition
    | conditionExpr op=OR conditionExpr                         # checkNestedCondition
    | conditionFunc                                             # checkConditionFunc
    | boolConst                                                 # checkConditionBoolConst
    ;

filterOnlyCondExp:
    '(' filterOnlyCondExp ')'                                   # prioritisedFilterExpr
    | valueArg op=(LT|GT|EQ|NEQ|LTE|GTE) valueArg               # simpleFilterExpr
    | filterOnlyCondExp op=AND filterOnlyCondExp                # composedSimpleFilterExpr
    | filterOnlyCondExp op=OR filterOnlyCondExp                 # composedSimpleFilterExpr
    ;

valueArg:
    ID
    | constant;

conditionFunc:
    contains
    ;

contains:  CONTAINS '(' nestedValueExpr ',' nestedValueExpr ')' # callContains;

intExpr:
    intExpr op=(ADD|SUB) intExpr        # intByAddSub
    | intExpr op=(MUL|DIV) intExpr      # intByMulDiv
    | INT                               # intByInt
    | INDEX                             # intByIndex
    | match                             # intByMatch
    | sequenceRef                       # intBySequenceRef
    | STRING                            # intByString
    | minusInt                          # intByMiusInt
    ;

transtion:
    daily
    | monthly
    ;

valueFunc:
    match
    | sum
    | condition
    | timestamp
    | hour
    | minute
    | count
    | distinctcount
    | date
    | now
    | concat
    | floor
    | ceil
    | datetime
    | dateformat
    | squarewave
    | random
    | intsToFloat
    | dint
    | pick
    | mod
    | find
    | max
    | min
    | minusInt
    | minusDouble
    | left
    | right
    | dayofweek
    | valueAt
    | stdevs
    | upper
    | lower
    | trim
    | text
    | mid
    | len
    | mathBit
    | indexSearchFunc
    | replace
    | replaceAll
    | cryptMd5
    | mathNormsdist
    | power
    | contains
    ;

indexSearchFunc:
    sumif
    | countif
    | distinctcountif
    | quartileif
    | maxif
    | minif
    | stdevsif
    | find
    | max
    | min
    | xlookup
    | xsumif
    | xcountif
    ;

indexExpr:
    intExpr                #getIndexByInt
    ;

sequence:
    ID '[' indexExpr '..' indexExpr ']'
    | ID
    ;

range:
    sequence                #callSequenceRange
    |ID '..' INT            #callArgsRange
    |ID ':' ID              #callArgsRangeInExcelWay
    ;

sequenceRef:
    ID '[' indexExpr ']'
    | ID
    ;

//If
condition:  IF '(' conditionExpr ',' expr ',' expr ')'                #callIf ;

//Time
now:        NOW '(' ')'                                               #callNow ;
timestamp:  TIMESTAMP '(' intExpr ')'                                 #callTimestamp ;
hour:       HOUR '(' ')'                                              #callHour ;
minute:     MINUTE '(' ')'                                            #callMinute ;
daily:      DAILY '(' ')'                                             #callDaily ;
monthly:    MONTHLY '(' ')'                                           #callMonthly ;

//Date
date:       DATE '(' expr (',' STRING)? ')'                           #callDate ;
datetime:   DATETIME '(' STRING (',' INT)? ')'                        #callDateTime ;
dateformat: DATEFORMAT '(' expr (',' STRING)? ')'                     #callDateFormat ;
dayofweek:  DAYOFWEEK '(' nestedValueExpr ')'                         #callDayOfWeek;

//Container
valueAt:    VALUEAT '(' nestedValueExpr ','  intExpr ')'              #callValueAt;

//Text
concat:     CONCAT '(' expr (',' expr)* ')'                           #callStringConcat ;
pick:       PICK '(' nestedValueExpr ',' STRING ',' INT ')'           #callPick;
left:       LEFT '(' nestedValueExpr (',' intExpr)? ')'               #callLeft;
right:      RIGHT '(' nestedValueExpr (',' intExpr)? ')'              #callRight;
upper:      UPPER '(' nestedValueExpr ')'                             #callUpper;
lower:      LOWER '(' nestedValueExpr ')'                             #callLower;
trim:       TRIM '(' nestedValueExpr ')'                              #callTrim;
replace:    REPLACE '(' nestedValueExpr ',' STRING ',' STRING ')'     #callReplace;
replaceAll: REPLACEALL '(' nestedValueExpr ',' STRING ',' STRING ')'  #callReplaceAll;
text:       TEXT'(' expr ')'                                          #callText;
mid:        MID'(' nestedValueExpr ',' nestedValueExpr ',' nestedValueExpr')' #callMid;
len:        LEN'(' nestedValueExpr ')'                                #callLength;

//Aggregation
distinctcount:      DISTINCTCOUNT '(' range ')'                       #callDistinctCount ;
countif:    COUNTIF '(' range ',' conditionExpr  ')'                  #callCountIf ;
distinctcountif:    DISTINCTCOUNTIF '(' range ',' conditionExpr ')'   #callDistinctCountif ;
match:      MATCH '(' expr ',' sequence ')'                           #callMatch ;
sum:        SUM '(' range ')'                                         #callSum ;
sumif:      SUMIF '(' range ',' conditionExpr  ')'                    #callSumIf ;
count:      COUNT '(' range ')'                                       #callCount ;
find:       FIND '(' ID ',' conditionExpr (',' INT)? ')'              #callFind;
max:        MAX '(' find ')'                                          #callMax;
min:        MIN '(' find ')'                                          #callMin;
minif:      MINIF '(' ID ',' conditionExpr  ')'                       #callMinIf;
maxif:      MAXIF '(' ID ',' conditionExpr  ')'                       #callMaxIf;
stdevs:     STDEVS '(' range ')'                                      #callStandardErrorSampled;
stdevsif:   STDEVSIF '(' range ',' conditionExpr  ')'                 #callStandardErrorSampledConditional;
quartileif: QUARTILEIF '(' ID ',' INT ',' conditionExpr ')'           #callQuartileIf;
xlookup: XLOOKUP '(' nestedValueExpr ',' ID ',' ID  ')' #callXLookup;
xsumif: XSUMIF '(' ID ',' filterOnlyCondExp  ')'      #callXSumif;
xcountif: XCOUNTIF '(' ID ',' filterOnlyCondExp  ')'  #callXCountif;

//Math
cryptMd5:   CRYPT_MD5 '(' nestedValueExpr (',' nestedValueExpr)* ')'  #callCryptMd5;
mod:        MOD '(' nestedValueExpr ',' nestedValueExpr')'            #callMod;
squarewave:     SQUAREWAVE  '(' intExpr ')'                           #callSquareWave;
random:     RANDOM '('')'                                             #callRandom;
intsToFloat: INTSTOFLOAT '(' intExpr ',' intExpr (',' INT)? ')'       #callInts2Float;
power:      POWER '(' nestedValueExpr ',' nestedValueExpr ')'         #callPower;
minusInt:   '-' INT                                                   #callMinusInt;
minusDouble:'-' DOUBLE                                                #callMinusDouble;
dint:       DINT '(' intExpr ',' intExpr ')'                          #callDInt;
floor:      FLOOR '(' expr ')'                                        #callFloor ;
ceil:       CEIL  '(' expr ')'                                        #callCeil ;
mathBit:    MATH_BIT '(' nestedValueExpr ',' nestedValueExpr ')'      #callMathBit;
mathNormsdist: MATH_NORMSDIST '(' nestedValueExpr ',' nestedValueExpr ',' nestedValueExpr ')'              #callMathNormsdist;

constant:
    numberConst             #constByNumber
    | boolConst             #constByBool
    | INDEX                 #constByIndex
    | STRING                #constByString
    | NULL                  #constByNull
;

numberConst:
    DOUBLE                  #constByDouble
    | INT                   #constByInt
;

boolConst:
    TRUE                    #constByTrue
    | FALSE                 #constByFalse
    ;

//Function Name
DATETIME: D A T E T I M E;
DATEFORMAT: D A T E F O R M A T;
MATCH: M A T C H;
SUM: S U M;
SUMIF: S U M I F;
IF: I F;
TIMESTAMP: T I M E S T A M P;
HOUR: H O U R;
MINUTE: M I N U T E;
DAILY: D A I L Y;
MONTHLY: M O N T H L Y;
COUNT: C O U N T;
DISTINCTCOUNT: D I S T I N C T C O U N T;
COUNTIF: C O U N T I F;
DISTINCTCOUNTIF: D I S T I N C T C O U N T I F;
FINDALL: F I N D A L L;
DATE: D A T E;
NOW: N O W;
INDICATOR: I N D I C A T O R;
CONCAT: C O N C A T;
FLOOR: F L O O R;
CEIL: C E I L;
FORM: F O R M;
VALUE: V A L U E;
LABEL: L A B E L;
SUBMITTIME: S U B M I T T I M E;
SUBMITUSER: S U B M I T U S E R;
INTSTOFLOAT: I N T S T O F L O A T;
DINT: D I N T;
SQUAREWAVE: S Q U A R E W A V E;
RANDOM: R A N D O M;
QUARTILEIF: Q U A R T I L E I F;
PICK: P I C K;
MOD: M O D;
FIND: F I N D;
MAX: M A X;
MIN: M I N;
MAXIF: M A X I F;
MINIF: M I N I F;
LEFT: L E F T;
RIGHT: R I G H T;
DAYOFWEEK: D A Y O F W E E K;
VALUEAT: V A L U E A T;
STDEVS: S T D E V S;
STDEVSIF: S T D E V S I F;
UPPER: U P P E R;
LOWER: L O W E R;
TRIM: T R I M;
TEXT : T E X T;
MID: M I D;
LEN: L E N;
CONTAINS: C O N T A I N S;
MATH_BIT: M A T H '.' B I T;
MATH_NORMSDIST: M A T H '.' N O R M S D I S T;
POWER: P O W E R;
TIMESPAN: T I M E S P A N;
REPLACE: R E P L A C E;
REPLACEALL: R E P L A C E A L L;
CRYPT_MD5: C R Y P T '.' M D '5';
TIME_TS: T I M E '.' T S;
SUBMITUSERID: S U B M I T U S E R I D;
SUBMITUSERACCOUNT: S U B M I T U S E R A C C O U N T;
IND_SUM: I N D '.' S U M;

//JSON
JSON_SELECT: J S O N '.' S E L E C T;

//Incremental Function Name
XLOOKUP: X L O O K U P;
XSUMIF: X S U M I F;
XCOUNTIF: X C O U N T I F;

//Keywords
NULL: N U L L;
TRUE: T R U E;
FALSE: F A L S E;
INDEX: I N D E X;
AND: A N D;
OR: O R;
MUL : '*' ;
DIV : '/' ;
ADD : '+' ;
SUB : '-' ;
GT  : '>';
GTE : '>=';
LT  : '<';
LTE : '<=';
EQ  : '=';
NEQ  : '<>';

//Const
INT : [0-9]+ ;
DOUBLE: INT '.' INT;
ID: ARG | ARG'.'ARG;
ARG: ('$')?[a-zA-Z]+([0-9]+)?;
STRING: '"' ('""' | ~["])* '"';

//Case Ignore
fragment A : [aA];
fragment B : [bB];
fragment C : [cC];
fragment D : [dD];
fragment E : [eE];
fragment F : [fF];
fragment G : [gG];
fragment H : [hH];
fragment I : [iI];
fragment J : [jJ];
fragment K : [kK];
fragment L : [lL];
fragment M : [mM];
fragment N : [nN];
fragment O : [oO];
fragment P : [pP];
fragment Q : [qQ];
fragment R : [rR];
fragment S : [sS];
fragment T : [tT];
fragment U : [uU];
fragment V : [vV];
fragment W : [wW];
fragment X : [xX];
fragment Y : [yY];
fragment Z : [zZ];

//Truncate Space
WS: [ \t\n\r]+ -> skip;
