//
// StNames.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class StNamesInput : MLFeatureProvider {

    /// an increment as 1 by 3 matrix of 16-bit floats
    public var var_: MLMultiArray

    public var featureNames: Set<String> { ["var"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "var" {
            return MLFeatureValue(multiArray: var_)
        }
        return nil
    }

    public init(var_: MLMultiArray) {
        self.var_ = var_
    }

    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #endif
    public convenience init(var_: MLShapedArray<Float16>) {
        self.init(var_: MLMultiArray(var_))
    }

}


/// Model Prediction Output Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class StNamesOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// add_0 as 1 by 3 matrix of 16-bit floats
    public var add_0: MLMultiArray {
        provider.featureValue(for: "add_0")!.multiArrayValue!
    }

    /// add_0 as 1 by 3 matrix of 16-bit floats
    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #endif
    public var add_0ShapedArray: MLShapedArray<Float16> {
        MLShapedArray<Float16>(add_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(add_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["add_0" : MLFeatureValue(multiArray: add_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}

/// Model Prediction State Type
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public final class StNamesState {
    public enum Name: String, CaseIterable {
        case out_ = "out"
    }

    let handle: MLState

    init(handle: MLState) {
        self.handle = handle
    }

    public func withMultiArray<R>(for stateName: Name, _ body: (MLMultiArray) throws -> R) rethrows -> R {
        try handle.withMultiArray(for: stateName.rawValue, body)
    }
}

/// Class for model loading and prediction
@available(macOS 15.0, iOS 18.0, tvOS 18.0, watchOS 11.0, visionOS 2.0, *)
public class StNames {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "StNames", withExtension:"mlmodelc")!
    }

    /**
        Construct StNames instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of StNames.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `StNames.urlOfModelInThisBundle` to create a MLModel object to pass-in.

        - parameters:
          - model: MLModel object
    */
    init(model: MLModel) {
        self.model = model
    }

    /**
        Construct a model with configuration

        - parameters:
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(configuration: MLModelConfiguration = MLModelConfiguration()) throws {
        try self.init(contentsOf: type(of:self).urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct StNames instance with explicit path to mlmodelc file
        - parameters:
           - modelURL: the file url of the model

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL) throws {
        try self.init(model: MLModel(contentsOf: modelURL))
    }

    /**
        Construct a model with URL of the .mlmodelc directory and configuration

        - parameters:
           - modelURL: the file url of the model
           - configuration: the desired model configuration

        - throws: an NSError object that describes the problem
    */
    public convenience init(contentsOf modelURL: URL, configuration: MLModelConfiguration) throws {
        try self.init(model: MLModel(contentsOf: modelURL, configuration: configuration))
    }

    /**
        Construct StNames instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<StNames, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct StNames instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> StNames {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct StNames instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<StNames, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(StNames(model: model)))
            }
        }
    }

    /**
        Construct StNames instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> StNames {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return StNames(model: model)
    }

    /**
        Make a new state.

        Core ML framework will allocate the state buffers declared in the model.

        The allocated state buffers are initialized to zeros. To initialize with different values, use `.withMultiArray(for:)` to get the mutable `MLMultiArray`-view to the state buffer.

        ```swift
        let state = model.makeState()
        state.withMultiArray(for: .out_) { stateMultiArray in
            stateMultiArray[0] = 0.42
        }
        ```
    */
    public func makeState() -> StNamesState {
        StNamesState(handle: model.makeState())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as StNamesInput
           - state: the state that the prediction will use and update.

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as StNamesOutput
    */
    public func prediction(input: StNamesInput, using state: StNamesState) throws -> StNamesOutput {
        try prediction(input: input, using: state, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as StNamesInput
           - state: the state that the prediction will use and update.
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as StNamesOutput
    */
    public func prediction(input: StNamesInput, using state: StNamesState, options: MLPredictionOptions) throws -> StNamesOutput {
        let outFeatures = try model.prediction(from: input, using: state.handle, options: options)
        return StNamesOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as StNamesInput
           - state: the state that the prediction will use and update.
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as StNamesOutput
    */
    public func prediction(input: StNamesInput, using state: StNamesState, options: MLPredictionOptions = MLPredictionOptions()) async throws -> StNamesOutput {
        let outFeatures = try await model.prediction(from: input, using: state.handle, options: options)
        return StNamesOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - var_: an increment as 1 by 3 matrix of 16-bit floats
            - state: the state that the prediction will use and update.

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as StNamesOutput
    */
    public func prediction(var_: MLMultiArray, using state: StNamesState) throws -> StNamesOutput {
        let input_ = StNamesInput(var_: var_)
        return try prediction(input: input_, using: state)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - var_: an increment as 1 by 3 matrix of 16-bit floats
            - state: the state that the prediction will use and update.

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as StNamesOutput
    */

    #if (os(macOS) || targetEnvironment(macCatalyst)) && arch(x86_64)
    @available(macOS, unavailable)
    @available(macCatalyst, unavailable)
    #endif
    public func prediction(var_: MLShapedArray<Float16>, using state: StNamesState) throws -> StNamesOutput {
        let input_ = StNamesInput(var_: var_)
        return try prediction(input: input_, using: state)
    }
}
