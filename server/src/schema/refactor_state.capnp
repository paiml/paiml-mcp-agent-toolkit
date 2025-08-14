@0x9a8f5c3b2e1d4789;

struct RefactorState {
    current @0 :State;
    history @1 :List(StateTransition);
    config @2 :RefactorConfig;
    targets @3 :List(Text);
    currentTargetIndex @4 :UInt32;
}

struct State {
    union {
        scan @0 :ScanState;
        analyze @1 :AnalyzeState;
        plan @2 :PlanState;
        refactor @3 :RefactorState;
        test @4 :TestState;
        lint @5 :LintState;
        emit @6 :EmitState;
        checkpoint @7 :CheckpointState;
        complete @8 :CompleteState;
    }
}

struct ScanState {
    targets @0 :List(Text);
}

struct AnalyzeState {
    current @0 :FileId;
}

struct PlanState {
    violations @0 :List(Violation);
}

struct RefactorState {
    operation @0 :RefactorOp;
}

struct TestState {
    command @0 :Text;
}

struct LintState {
    strict @0 :Bool;
}

struct EmitState {
    payload @0 :DefectPayload;
}

struct CheckpointState {
    reason @0 :Text;
}

struct CompleteState {
    summary @0 :Summary;
}

struct StateTransition {
    fromState @0 :State;
    toState @1 :State;
    timestamp @2 :UInt64;
    metricsBefore @3 :MetricSet;
    metricsAfter @4 :MetricSet;
    appliedRefactor @5 :RefactorOp;
}

struct RefactorConfig {
    targetComplexity @0 :UInt16;
    removeSatd @1 :Bool;
    maxFunctionLines @2 :UInt32;
    thresholds @3 :Thresholds;
    strategies @4 :RefactorStrategies;
    parallelWorkers @5 :UInt32;
    memoryLimitMb @6 :UInt32;
    batchSize @7 :UInt32;
    priorityExpression @8 :Text;
    autoCommitTemplate @9 :Text;
}

struct Thresholds {
    cyclomaticWarn @0 :UInt16;
    cyclomaticError @1 :UInt16;
    cognitiveWarn @2 :UInt16;
    cognitiveError @3 :UInt16;
    tdgWarn @4 :Float32;
    tdgError @5 :Float32;
}

struct RefactorStrategies {
    preferFunctional @0 :Bool;
    useEarlyReturns @1 :Bool;
    extractHelpers @2 :Bool;
}

struct MetricSet {
    cyclomaticComplexity @0 :UInt16;
    cognitiveComplexity @1 :UInt16;
    tdgScore @2 :Float32;
    deadCode @3 :List(Bool);
    satdCount @4 :UInt32;
    provability @5 :Float32;
}

struct RefactorOp {
    union {
        extractFunction @0 :ExtractFunction;
        flattenNesting @1 :FlattenNesting;
        replaceHashMap @2 :ReplaceHashMap;
        removeSatd @3 :RemoveSatd;
        simplifyExpression @4 :SimplifyExpression;
    }
}

struct ExtractFunction {
    name @0 :Text;
    start @1 :BytePos;
    end @2 :BytePos;
    params @3 :List(Text);
}

struct FlattenNesting {
    function @0 :Text;
    strategy @1 :NestingStrategy;
}

enum NestingStrategy {
    earlyReturn @0;
    extractCondition @1;
    guardClause @2;
    streamChain @3;
}

struct ReplaceHashMap {
    imports @0 :List(Text);
    replacements @1 :List(Replacement);
}

struct Replacement {
    from @0 :Text;
    to @1 :Text;
}

struct RemoveSatd {
    location @0 :Location;
    fix @1 :SatdFix;
}

struct SimplifyExpression {
    expr @0 :Text;
    simplified @1 :Text;
}

struct BytePos {
    byte @0 :UInt32;
    line @1 :UInt32;
    column @2 :UInt32;
}

struct Location {
    file @0 :Text;
    line @1 :UInt32;
    column @2 :UInt32;
}

struct SatdFix {
    union {
        remove @0 :Void;
        replace @1 :Text;
        implement @2 :Text;
    }
}

struct FileId {
    path @0 :Text;
    hash @1 :UInt64;
}

struct Violation {
    violationType @0 :ViolationType;
    location @1 :Location;
    severity @2 :Severity;
    description @3 :Text;
    suggestedFix @4 :RefactorOp;
}

enum ViolationType {
    highComplexity @0;
    deepNesting @1;
    longFunction @2;
    selfAdmittedTechDebt @3;
    deadCode @4;
    poorNaming @5;
}

enum Severity {
    low @0;
    medium @1;
    high @2;
    critical @3;
}

struct DefectPayload {
    fileHash @0 :UInt64;
    tdgScore @1 :Float32;
    cyclomaticComplexity @2 :UInt16;
    cognitiveComplexity @3 :UInt16;
    deadSymbols @4 :UInt32;
    timestamp @5 :UInt64;
    severityFlags @6 :UInt8;
    refactorAvailable @7 :Bool;
    refactorType @8 :RefactorType;
    estimatedImprovement @9 :Float32;
}

enum RefactorType {
    none @0;
    extractFunction @1;
    flattenNesting @2;
    simplifyLogic @3;
    removeDeadCode @4;
}

struct Summary {
    filesProcessed @0 :UInt32;
    refactorsApplied @1 :UInt32;
    complexityReduction @2 :Float32;
    satdRemoved @3 :UInt32;
    totalTimeSeconds @4 :UInt64;
}