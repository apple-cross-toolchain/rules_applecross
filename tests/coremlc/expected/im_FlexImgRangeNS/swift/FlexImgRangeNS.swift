//
// FlexImgRangeNS.swift
//
// This file was automatically generated and should not be edited.
//

import CoreML


/// Model Prediction Input Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class FlexImgRangeNSInput : MLFeatureProvider {

    /// img as color (kCVPixelFormatType_32BGRA) image buffer, 10 pixels wide by 6 pixels high
    public var img: CVPixelBuffer

    public var featureNames: Set<String> { ["img"] }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        if featureName == "img" {
            return MLFeatureValue(pixelBuffer: img)
        }
        return nil
    }

    public init(img: CVPixelBuffer) {
        self.img = img
    }

    public convenience init(imgWith img: CGImage) throws {
        self.init(img: try MLFeatureValue(cgImage: img, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!)
    }

    public convenience init(imgAt img: URL) throws {
        self.init(img: try MLFeatureValue(imageAt: img, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!)
    }

    public func setImg(with img: CGImage) throws  {
        self.img = try MLFeatureValue(cgImage: img, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

    public func setImg(with img: URL) throws  {
        self.img = try MLFeatureValue(imageAt: img, pixelsWide: 10, pixelsHigh: 6, pixelFormatType: kCVPixelFormatType_32ARGB, options: nil).imageBufferValue!
    }

}


/// Model Prediction Output Type
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class FlexImgRangeNSOutput : MLFeatureProvider {

    /// Source provided by CoreML
    private let provider : MLFeatureProvider

    /// reduce_mean_0 as 1 by 3 matrix of floats
    public var reduce_mean_0: MLMultiArray {
        provider.featureValue(for: "reduce_mean_0")!.multiArrayValue!
    }

    /// reduce_mean_0 as 1 by 3 matrix of floats
    public var reduce_mean_0ShapedArray: MLShapedArray<Float> {
        MLShapedArray<Float>(reduce_mean_0)
    }

    public var featureNames: Set<String> {
        provider.featureNames
    }

    public func featureValue(for featureName: String) -> MLFeatureValue? {
        provider.featureValue(for: featureName)
    }

    public init(reduce_mean_0: MLMultiArray) {
        self.provider = try! MLDictionaryFeatureProvider(dictionary: ["reduce_mean_0" : MLFeatureValue(multiArray: reduce_mean_0)])
    }

    public init(features: MLFeatureProvider) {
        self.provider = features
    }
}


/// Class for model loading and prediction
@available(macOS 13.0, iOS 16.0, tvOS 16.0, watchOS 9.0, visionOS 1.0, *)
public class FlexImgRangeNS {
    public let model: MLModel

    /// URL of model assuming it was installed in the same bundle as this class
    class var urlOfModelInThisBundle : URL {
        let bundle = Bundle(for: self)
        return bundle.url(forResource: "FlexImgRangeNS", withExtension:"mlmodelc")!
    }

    /**
        Construct FlexImgRangeNS instance with an existing MLModel object.

        Usually the application does not use this initializer unless it makes a subclass of FlexImgRangeNS.
        Such application may want to use `MLModel(contentsOfURL:configuration:)` and `FlexImgRangeNS.urlOfModelInThisBundle` to create a MLModel object to pass-in.

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
        Construct FlexImgRangeNS instance with explicit path to mlmodelc file
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
        Construct FlexImgRangeNS instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<FlexImgRangeNS, Error>) -> Void) {
        load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration, completionHandler: handler)
    }

    /**
        Construct FlexImgRangeNS instance asynchronously with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - configuration: the desired model configuration
    */
    public class func load(configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> FlexImgRangeNS {
        try await load(contentsOf: self.urlOfModelInThisBundle, configuration: configuration)
    }

    /**
        Construct FlexImgRangeNS instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
          - handler: the completion handler to be called when the model loading completes successfully or unsuccessfully
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration(), completionHandler handler: @escaping (Swift.Result<FlexImgRangeNS, Error>) -> Void) {
        MLModel.load(contentsOf: modelURL, configuration: configuration) { result in
            switch result {
            case .failure(let error):
                handler(.failure(error))
            case .success(let model):
                handler(.success(FlexImgRangeNS(model: model)))
            }
        }
    }

    /**
        Construct FlexImgRangeNS instance asynchronously with URL of the .mlmodelc directory with optional configuration.

        Model loading may take time when the model content is not immediately available (e.g. encrypted model). Use this factory method especially when the caller is on the main thread.

        - parameters:
          - modelURL: the URL to the model
          - configuration: the desired model configuration
    */
    public class func load(contentsOf modelURL: URL, configuration: MLModelConfiguration = MLModelConfiguration()) async throws -> FlexImgRangeNS {
        let model = try await MLModel.load(contentsOf: modelURL, configuration: configuration)
        return FlexImgRangeNS(model: model)
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as FlexImgRangeNSInput

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as FlexImgRangeNSOutput
    */
    public func prediction(input: FlexImgRangeNSInput) throws -> FlexImgRangeNSOutput {
        try prediction(input: input, options: MLPredictionOptions())
    }

    /**
        Make a prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as FlexImgRangeNSInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as FlexImgRangeNSOutput
    */
    public func prediction(input: FlexImgRangeNSInput, options: MLPredictionOptions) throws -> FlexImgRangeNSOutput {
        let outFeatures = try model.prediction(from: input, options: options)
        return FlexImgRangeNSOutput(features: outFeatures)
    }

    /**
        Make an asynchronous prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - input: the input to the prediction as FlexImgRangeNSInput
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as FlexImgRangeNSOutput
    */
    @available(macOS 14.0, iOS 17.0, tvOS 17.0, watchOS 10.0, visionOS 1.0, *)
    public func prediction(input: FlexImgRangeNSInput, options: MLPredictionOptions = MLPredictionOptions()) async throws -> FlexImgRangeNSOutput {
        let outFeatures = try await model.prediction(from: input, options: options)
        return FlexImgRangeNSOutput(features: outFeatures)
    }

    /**
        Make a prediction using the convenience interface

        It uses the default function if the model has multiple functions.

        - parameters:
            - img: color (kCVPixelFormatType_32BGRA) image buffer, 10 pixels wide by 6 pixels high

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as FlexImgRangeNSOutput
    */
    public func prediction(img: CVPixelBuffer) throws -> FlexImgRangeNSOutput {
        let input_ = FlexImgRangeNSInput(img: img)
        return try prediction(input: input_)
    }

    /**
        Make a batch prediction using the structured interface

        It uses the default function if the model has multiple functions.

        - parameters:
           - inputs: the inputs to the prediction as [FlexImgRangeNSInput]
           - options: prediction options

        - throws: an NSError object that describes the problem

        - returns: the result of the prediction as [FlexImgRangeNSOutput]
    */
    public func predictions(inputs: [FlexImgRangeNSInput], options: MLPredictionOptions = MLPredictionOptions()) throws -> [FlexImgRangeNSOutput] {
        let batchIn = MLArrayBatchProvider(array: inputs)
        let batchOut = try model.predictions(from: batchIn, options: options)
        var results : [FlexImgRangeNSOutput] = []
        results.reserveCapacity(inputs.count)
        for i in 0..<batchOut.count {
            let outProvider = batchOut.features(at: i)
            let result =  FlexImgRangeNSOutput(features: outProvider)
            results.append(result)
        }
        return results
    }
}
