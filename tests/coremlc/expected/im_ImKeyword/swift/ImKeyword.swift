//
// ImKeyword.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImKeywordInput : MLFeatureProvider {

    /// self as color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
    public var self_: CVPixelBuffer

    /// var as 1 by 3 matrix of floats
    public var var_: MLMultiArray

    public var featureNames: Set<String> { ["self", "var"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "self" {
            return MLFeatureValue(pixelBuffer: self_)
        }
        if featureName == "var" {
            return MLFeatureValue(multiArray: var_)
        }
        return nil
    }

    public init(self_: CVPixelBuffer, var_: MLMultiArray) {
        self.self_ = self_
        self.var_ = var_
    }

    public convenience init(self_: CVPixelBuffer, var_: MLShapedArray<Float>) {
        self.init(self_: self_, var_: MLMultiArray(var_))
    }

    public convenience init(self_With self_: CGImage, var_: MLMultiArray) throws {
        self.init(self_: try MLFeatureValue(cgImage: self_, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!, var_: var_)
    }

    public convenience init(self_With self_: CGImage, var_: MLShapedArray<Float>) throws {
        self.init(self_: try MLFeatureValue(cgImage: self_, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!, var_: MLMultiArray(var_))
    }

    public convenience init(self_At self_: URL, var_: MLMultiArray) throws {
        self.init(self_: try MLFeatureValue(imageAt: self_, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!, var_: var_)
    }

    public convenience init(self_At self_: URL, var_: MLShapedArray<Float>) throws {
        self.init(self_: try MLFeatureValue(imageAt: self_, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!, var_: MLMultiArray(var_))
    }

    public func setSelf_(with self_: CGImage) throws  {
        self.self_ = try MLFeatureValue(cgImage: self_, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

    public func setSelf_(with self_: URL) throws  {
        self.self_ = try MLFeatureValue(imageAt: self_, pixelsWide: 4, pixelsHigh: 4, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImKeywordOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// reshape_0 as 1 element vector of floats
    public var reshape_0: MLMultiArray {
        provider.featureValue(for: "reshape_0")!.multiArrayValue!
    }

    /// reshape_0 as 1 element vector of floats
    public var reshape_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(reshape_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(reshape_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["reshape_0" : MLFeatureValue(multiArray: reshape_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class ImKeyword {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "ImKeyword", withExtension:"mlmodelc")!
    }

    /**
        Construct ImKeyword instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of ImKeyword.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `ImKeyword.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct ImKeyword instance with explicit path to mlmodelc file
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
        Construct ImKeyword instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ImKeyword, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct ImKeyword instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ImKeyword {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct ImKeyword instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<ImKeyword, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(ImKeyword(model: model)))
            }
        }
    }

    /**
        Construct ImKeyword instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> ImKeyword {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return ImKeyword(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImKeywordInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImKeywordOutput
    */
    public func prediction(input: ImKeywordInput) throws -> ImKeywordOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImKeywordInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImKeywordOutput
    */
    public func prediction(input: ImKeywordInput, options: MLPredictionOptions) throws -> ImKeywordOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return ImKeywordOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as ImKeywordInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImKeywordOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: ImKeywordInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> ImKeywordOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return ImKeywordOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - self_: color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
            - var_: 1 by 3 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImKeywordOutput
    */
    public func prediction(self_: CVPixelBuffer, var_: MLMultiArray) throws -> ImKeywordOutput {
        let input_ = ImKeywordInput(self_: self_, var_: var_)
        return try prediction(input: input_)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - self_: color (kCVPixelFormatType_32BGRA) image buffer, 4 pixels wide by 4 pixels high
            - var_: 1 by 3 matrix of floats

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as ImKeywordOutput
    */

    public func prediction(self_: CVPixelBuffer, var_: MLShapedArray<Float>) throws -> ImKeywordOutput {
        let input_ = ImKeywordInput(self_: self_, var_: var_)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [ImKeywordInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [ImKeywordOutput]
    */
    public func predictions(inputs: [ImKeywordInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [ImKeywordOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [ImKeywordOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  ImKeywordOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
