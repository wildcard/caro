/**
 * AsciiMorph - Multi-instance wrapper
 * Based on Tim Holman's ascii-morph library
 * Modified to support multiple instances on the same page
 */

(function() {
  'use strict';

  // Factory function to create isolated instances
  function createAsciiMorph(element, canvasSize) {
    var renderedData = [];
    var myTimeout = null;

    function repeat(pattern, count) {
      if (count < 1) return '';
      var result = '';
      while (count > 1) {
        if (count & 1) result += pattern;
        count >>= 1, pattern += pattern;
      }
      return result + pattern;
    }

    function squareOutData(data) {
      var i;
      var renderDimensions = {
        x: 0,
        y: data.length
      };

      for (i = 0; i < data.length; i++) {
        if (data[i].length > renderDimensions.x) {
          renderDimensions.x = data[i].length;
        }
      }

      for (i = 0; i < data.length; i++) {
        if (data[i].length < renderDimensions.x) {
          data[i] = (data[i] + repeat(' ', renderDimensions.x - data[i].length));
        }
      }

      var paddings = {
        x: Math.floor((canvasSize.x - renderDimensions.x) / 2),
        y: Math.floor((canvasSize.y - renderDimensions.y) / 2)
      };

      var paddedData = [];
      var verticalPadding = repeat(' ', canvasSize.x);

      for (i = 0; i < paddings.y; i++) {
        paddedData.push(verticalPadding);
      }

      for (i = 0; i < data.length; i++) {
        paddedData.push(repeat(' ', paddings.x) + data[i] + repeat(' ', paddings.x));
      }

      for (i = 0; i < paddings.y; i++) {
        paddedData.push(verticalPadding);
      }

      return paddedData;
    }

    function getMorphedFrame(fromFrame, toFrame, interpolationPoint) {
      var interpolatedFrame = [];
      var fromLine;
      var toLine;
      var midPoint;
      var i;

      for (i = 0; i < fromFrame.length; i++) {
        fromLine = fromFrame[i];
        toLine = toFrame[i];
        midPoint = Math.floor(toLine.length * interpolationPoint);
        interpolatedFrame.push(fromLine.substring(0, midPoint) + toLine.substring(midPoint));
      }
      return interpolatedFrame;
    }

    function renderSquareData(data) {
      var ourData = squareOutData(data.slice());
      renderedData = ourData;
      var renderOutput = '';

      for (var i = 0; i < ourData.length; i++) {
        renderOutput += (ourData[i] + '\n');
      }

      element.textContent = renderOutput;
    }

    function render(data) {
      clearTimeout(myTimeout);
      renderSquareData(data);
    }

    function morph(data, frameDelay) {
      clearTimeout(myTimeout);
      frameDelay = frameDelay || 50;

      var frames = [];
      var interpolationStep = 0.05;

      for (var i = 0; i <= 1; i += interpolationStep) {
        frames.push(getMorphedFrame(renderedData, squareOutData(data.slice()), i));
      }

      var frameIndex = 0;

      function animateFrames() {
        if (frameIndex < frames.length) {
          renderedData = frames[frameIndex];
          element.textContent = frames[frameIndex].join('\n');
          frameIndex++;
          myTimeout = setTimeout(animateFrames, frameDelay);
        }
      }

      animateFrames();
    }

    // Return instance methods
    return {
      render: render,
      morph: morph
    };
  }

  // Expose factory function
  window.createAsciiMorph = createAsciiMorph;
})();
